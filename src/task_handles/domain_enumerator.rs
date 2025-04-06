use crate::libs::args::Args;
use crate::{libs, models};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn handle(
    scan_id: String,
    sqlite_pool: sqlx::Pool<sqlx::Sqlite>,
    is_paused: Arc<AtomicBool>,
    args: Args,
) {
    let resolver = match libs::dns::create_resolver() {
        Ok(resolver) => resolver,
        Err(e) => {
            let _ = libs::sqlite::insert_log(
                scan_id.clone(),
                "error".to_string(),
                format!("Error creating DNS resolver: {}", e),
                &sqlite_pool,
                args.log_level.to_string(),
            )
            .await;
            return;
        }
    };

    let user_agent = match args.active_random_user_agent {
        true => libs::rng::user_agent(),
        false => args.active_user_agent.clone(),
    };
    let client = match reqwest::Client::builder().user_agent(user_agent).build() {
        Ok(client) => client,
        Err(e) => {
            let _ = libs::sqlite::insert_log(
                scan_id.clone(),
                "error".to_string(),
                format!("Error creating HTTP client, terminating task: {e}"),
                &sqlite_pool,
                args.log_level.to_string(),
            )
            .await;
            return;
        }
    };

    loop {
        // If paused, wait and retry
        tokio::time::sleep(std::time::Duration::from_millis(args.interval)).await;
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
                            args.log_level.to_string(),
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
            let scan_result = crate::scanners::active_scan::execute(&resolver, &client, &args, &test_domain).await;

            for negative_result in scan_result.negatives {
                let _ = libs::sqlite::insert_log(
                    scan_id.clone(),
                    negative_result.level,
                    negative_result.description,
                    &sqlite_pool,
                    args.log_level.to_string(),
                )
                .await;
            }

            let status = match scan_result.found {
                true => {
                    let _ = libs::sqlite::insert_log(
                        scan_id.clone(),
                        "info".to_string(),
                        format!("Found: {}", test_domain),
                        &sqlite_pool,
                        args.log_level.to_string(),
                    )
                    .await;
                    "found"
                }
                false => "not found",
            };

            let update_query = format!(
                "UPDATE {scan_id} SET status = '{status}', last_scanned_on = CURRENT_TIMESTAMP, source = '{source}' WHERE id = {result_id};",
                scan_id = scan_id,
                status = status,
                source = scan_result.source,
                result_id = result.id
            );

            if let Err(e) = sqlx::query(&update_query).execute(&sqlite_pool).await {
                let _ = libs::sqlite::insert_log(
                    scan_id.clone(),
                    "error".to_string(),
                    format!("Error updating domain status: {}", e),
                    &sqlite_pool,
                    args.log_level.to_string(),
                )
                .await;
            }
        }
    }
}
