use crate::{libs};
use std::sync::{Arc, RwLock};
use crate::libs::args::Args;
use crate::libs::sqlite::{Log, ScanResults};

pub async fn handle(
    scan_id: String,
    sqlite_pool: sqlx::Pool<sqlx::Sqlite>,
    results_arc: Arc<RwLock<ScanResults>>,
    logs_arc: Arc<RwLock<Vec<Log>>>,
    args: Args,
) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let completed = results_arc.read().unwrap().found.len() as i32 + results_arc.read().unwrap().not_found >= results_arc.read().unwrap().total;

        if completed { continue; }

        let get_progress = libs::sqlite::get_results(
            scan_id.clone(), sqlite_pool.clone()
        ).await;

        match get_progress {
            Ok(results) => {
                let mut results_writer = results_arc.write().unwrap();
                results_writer.found = results.found;
                results_writer.not_found = results.not_found;
                results_writer.total = results.total;
            }
            Err(e) => {
                let _ = libs::sqlite::insert_log(
                    scan_id.clone(),
                    "error".to_string(),
                    format!("Error getting progress: {}", e),
                    &sqlite_pool,
                    args.log_level.to_string(),
                ).await;
            }
        }

        let get_logs = libs::sqlite::get_logs(
            scan_id.clone(),
            "debug".to_string(),
            sqlite_pool.clone(),
        ).await;

        match get_logs {
            Ok(logs) => {
                let mut logs_writer = logs_arc.write().unwrap();
                logs_writer.clear();
                logs_writer.extend(logs);
            }
            Err(e) => {
                let _ = libs::sqlite::insert_log(
                    scan_id.clone(),
                    "error".to_string(),
                    format!("Error getting logs: {}", e),
                    &sqlite_pool,
                    args.log_level.to_string(),
                ).await;
            }
        }


    }
}
