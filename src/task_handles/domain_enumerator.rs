use crate::{libs, models};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn handle(
    scan_id: String,
    sqlite_pool: sqlx::Pool<sqlx::Sqlite>,
    is_paused: Arc<AtomicBool>,
    interval: u64,
) {
    let resolver = match libs::dns::create_resolver() {
        Ok(resolver) => resolver,
        Err(e) => {
            let _ = libs::sqlite::insert_log(
                scan_id.clone(),
                "error".to_string(),
                format!("Error creating DNS resolver: {}", e),
                &sqlite_pool,
            )
            .await;
            return;
        }
    };
    let client = reqwest::Client::new();

    loop {
        // If paused, wait and retry
        tokio::time::sleep(std::time::Duration::from_millis(interval)).await;
        if is_paused.load(Ordering::Relaxed) {
            continue;
        }

        let query = format!(
            "WITH selected AS (
                SELECT ROWID, domain FROM {scan_id} WHERE status = 'queued' ORDER BY ROWID LIMIT 1
            )
            UPDATE {scan_id}
            SET status = 'scanning', last_scan_started_on = CURRENT_TIMESTAMP
            WHERE ROWID IN (SELECT ROWID FROM selected)
            RETURNING id, domain, subdomain, max_retries;",
            scan_id = scan_id
        );

        let fetch_workload = sqlx::query_as::<_, models::result::SlimResult>(&query)
            .fetch_all(&sqlite_pool)
            .await;

        let queued_workload = match fetch_workload {
            Ok(queued_wl) if !queued_wl.is_empty() => queued_wl,
            _ => {
                // No queued domains, check if anything is still scanning
                let fetch_scanning = sqlx::query_as::<_, models::result::SlimResult>(
                    &format!("SELECT id, domain, subdomain, max_retries FROM {scan_id} WHERE status = 'scanning'", scan_id = scan_id)
                ).fetch_all(&sqlite_pool).await;

                match fetch_scanning {
                    Ok(scanning) if scanning.is_empty() => break, // Exit if nothing is scanning
                    Ok(_) => continue, // If something is scanning, keep looping
                    Err(e) => {
                        let _ = libs::sqlite::insert_log(
                            scan_id.clone(),
                            "error".to_string(),
                            format!("Error fetching scanning domains: {}", e),
                            &sqlite_pool,
                        )
                        .await;
                        continue;
                    }
                }
            }
        };

        for result in queued_workload {
            if is_paused.load(Ordering::Relaxed) {
                break; // Stop processing if paused
            }

            let test_domain = format!("{}.{}", result.subdomain, result.domain);
            let check_domain_exists = libs::domain_exists(&resolver, &client, &test_domain).await;

            let domain_exists = match check_domain_exists {
                Ok(exists) => exists,
                Err(e) => {
                    let _ = libs::sqlite::insert_log(
                        scan_id.clone(),
                        "error".to_string(),
                        format!("Error checking domain existence: {}", e.join(", ")),
                        &sqlite_pool,
                    )
                    .await;
                    false
                }
            };

            let status = if domain_exists { "found" } else { "not found" };

            let update_query = format!(
                "UPDATE {scan_id} SET status = '{status}', last_scanned_on = CURRENT_TIMESTAMP WHERE id = {result_id};",
                scan_id = scan_id,
                status = status,
                result_id = result.id
            );

            if let Err(e) = sqlx::query(&update_query).execute(&sqlite_pool).await {
                let _ = libs::sqlite::insert_log(
                    scan_id.clone(),
                    "error".to_string(),
                    format!("Error updating domain status: {}", e),
                    &sqlite_pool,
                )
                .await;
            }
        }
    }
}
