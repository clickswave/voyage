mod export_results;
mod libs;
mod models;
mod scanners;
mod task_handles;
mod tui;

use std::env;
use std::process::exit;
use std::sync::{Arc, RwLock};
use task_handles::domain_enumerator;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // parse arguments
    let args = libs::args::parse();
    // find os
    let os = env::consts::OS;
    let username = whoami::username();
    // determine db path
    let db_path = match os {
        "linux" => format!("/home/{}/.local/share/clickswave/voyage", username),
        "windows" => format!(r"C:\Users\{}\AppData\Roaming\clickswave\voyage", username),
        "macos" => format!(
            "/Users/{}/Library/Application Support/clickswave/voyage",
            username
        ),
        _ => ".".to_string(),
    };
    if args.recreate_database {
        // delete db path if exists
        if std::path::Path::new(db_path.as_str()).exists() {
            std::fs::remove_dir_all(db_path.clone()).unwrap_or_else(|_| {
                eprintln!("[ERROR] Error deleting db path");
                exit(1);
            });
        }
    };
    // create db path if not exists
    if !std::path::Path::new(db_path.as_str()).exists() {
        std::fs::create_dir_all(db_path.clone()).unwrap_or_else(|_| {
            eprintln!("[ERROR] Error creating db path");
            exit(1);
        });
    }
    let wordlist_path = match args.wordlist_path.is_empty() {
        true => {
            eprintln!("[WARN] No wordlist specified");
            exit(1);
        }
        false => args.wordlist_path.as_str(),
    };
    // initialize sqlite db
    let sqlite_pool = libs::sqlite::init(db_path).await?;
    // get wordlist hash
    let wordlist_hash = libs::wordlist::sha512(wordlist_path).await?;
    let config = models::scan::Config {
        domains: args.domain.clone(),
        wordlist_hash,
    };
    // convert config struct to json string
    let config_json = match serde_json::to_string(&config) {
        Ok(config_json) => config_json,
        Err(e) => {
            eprintln!("[ERROR] Error serializing config: {}", e);
            exit(1);
        }
    };
    // generate config hash
    let config_hash = match libs::sha::sha512(config_json.clone()).await {
        Ok(config_hash) => config_hash,
        Err(e) => {
            eprintln!("[ERROR] Error hashing config: {}", e);
            exit(1);
        }
    };
    let cached_scan =
        sqlx::query_as::<_, models::scan::Scan>("SELECT * FROM scans WHERE config_hash = ?")
            .bind(config_hash.clone())
            .fetch_optional(&sqlite_pool)
            .await?;

    // check if scan is cached, if not create a new scan, return progress position
    let mut scan = match cached_scan {
        None => {
            let scan_id = libs::rng::scan_id();
            let create_scan = sqlx::query_as::<_, models::scan::Scan>(
                "INSERT INTO scans
        (id, config_hash, config, status, no_banner, launch_delay)
    VALUES (?, ?, ?, ?, ?, ?)
    RETURNING *",
            )
            .bind(scan_id.clone())
            .bind(config_hash.clone())
            .bind(config_json.clone())
            .bind("scan_created")
            .bind(false)
            .bind(0)
            .fetch_one(&sqlite_pool)
            .await;
            match create_scan {
                Ok(scan) => {
                    libs::sqlite::insert_log(
                        scan.id.clone(),
                        "info".to_string(),
                        format!("Scan created with id: {}", scan.id),
                        &sqlite_pool,
                        args.log_level.to_string()
                    )
                    .await?;
                    scan
                }
                Err(e) => {
                    eprintln!("[ERROR] Error creating scan: {}", e);
                    exit(1);
                }
            }
        }
        Some(scan) => {
            if args.fresh_start {
                // set row in scans table to 'scan_created'
                let fresh_start =
                    libs::sqlite::fresh_start(scan.id.clone(), sqlite_pool.clone()).await;
                match fresh_start {
                    Ok(fresh_scan) => {
                        libs::sqlite::insert_log(
                            scan.id.clone(),
                            "info".to_string(),
                            format!("Scan {} set to fresh start", scan.id),
                            &sqlite_pool,
                            args.log_level.to_string()
                        )
                        .await?;
                        fresh_scan
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Error setting scan to fresh start: {}", e);
                        exit(1);
                    }
                }
            } else {
                libs::sqlite::insert_log(
                    scan.id.clone(),
                    "info".to_string(),
                    format!("Scan {} already exists", scan.id),
                    &sqlite_pool,
                    args.log_level.to_string()
                )
                .await?;
                scan
            }
        }
    };

    // ---------------------------------------------
    // next step
    // ---------------------------------------------
    // scan progress at scan created, create workload table
    if scan.status == "scan_created" {
        let create_workload_table =
            libs::sqlite::create_workload_table(scan.id.clone(), sqlite_pool.clone()).await;
        if let Err(e) = create_workload_table {
            eprintln!("[ERROR] Error creating workload table: {}", e);
            exit(1);
        }
        libs::sqlite::insert_log(
            scan.id.clone(),
            "info".to_string(),
            "Workload table created".to_string(),
            &sqlite_pool,
            args.log_level.to_string()
        )
        .await?;
        scan.status = "workload_table_created".to_string();
    }
    //if scan progress = workload_table_created, populate basic workload
    if scan.status == "workload_table_created" {
        let config: models::scan::Config = match serde_json::from_str(scan.config.as_str()) {
            Ok(user) => user,
            Err(e) => {
                eprintln!("[ERROR] Error parsing config: {}", e);
                exit(1);
            }
        };

        for domain in config.domains {
            if !args.disable_passive_enum {
                let passive_scan_results = scanners::passive_scan::execute(&domain, args.clone()).await?;

                let populate_passive_results = libs::sqlite::populate_passive_scan_results(
                    scan.id.clone(),
                    sqlite_pool.clone(),
                    passive_scan_results,
                    domain.clone()
                )
                .await;

                match populate_passive_results {
                    Ok(_) => {
                        libs::sqlite::insert_log(
                            scan.id.clone(),
                            "info".to_string(),
                            format!("Passive results populated for domain: {}", domain),
                            &sqlite_pool,
                            args.log_level.to_string()
                        )
                        .await?;
                        scan.status = "passive_results_populated".to_string();
                    }
                    Err(e) => {
                        libs::sqlite::insert_log(
                            scan.id.clone(),
                            "error".to_string(),
                            format!("Could not populate passive results: {}", e),
                            &sqlite_pool,
                            args.log_level.to_string()
                        )
                        .await?;
                    }
                }
            }
        }
    }

    if scan.status == "passive_results_populated"
        || (args.disable_passive_enum && scan.status == "workload_table_created")
    {
        if !args.disable_active_enum {
            for domain in config.domains {
                let populate_basic_workload = libs::sqlite::populate_basic_workload(
                    scan.id.clone(),
                    domain.clone(),
                    wordlist_path.to_string(),
                    sqlite_pool.clone(),
                )
                    .await;
                if let Err(e) = populate_basic_workload {
                    eprintln!("[ERROR] Error populating basic workload: {}", e);
                    exit(1);
                }
                libs::sqlite::insert_log(
                    scan.id.clone(),
                    "info".to_string(),
                    format!("Basic workload populated for domain: {}", domain),
                    &sqlite_pool,
                    args.log_level.to_string()
                )
                    .await?;
                scan.status = "basic_workload_populated".to_string();
            }
        }
    }

    // reset results from 'scanning' to 'queued'
    // scanning status has been touched by a thread but was halted before it could be processed
    libs::sqlite::reset_halted_scans(scan.id.clone(), sqlite_pool.clone()).await?;
    libs::sqlite::insert_log(
        scan.id.clone(),
        "info".to_string(),
        "Previously halted items (if any) has been reset".to_string(),
        &sqlite_pool,
        args.log_level.to_string()
    )
    .await?;

    let is_paused = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let results_arc = Arc::new(RwLock::new(
        libs::sqlite::get_results(scan.id.clone(), sqlite_pool.clone()).await?,
    ));
    let logs_arc = Arc::new(RwLock::new(
        libs::sqlite::get_logs(scan.id.clone(), "debug".to_string(), sqlite_pool.clone(), ).await?,
    ));
    // setup threads
    let mut threads = vec![];
    // push scanner threads
    for index in 0..args.tasks.clone() {
        let thread = tokio::spawn(domain_enumerator::handle(
            scan.id.clone(),
            sqlite_pool.clone(),
            is_paused.clone(),
            args.clone(),
        ));
        threads.push(thread);
        libs::sqlite::insert_log(
            scan.id.clone(),
            "debug".to_string(),
            format!("Task {} spawned", index + 1),
            &sqlite_pool,
            args.log_level.to_string()
        )
        .await?;
    }
    libs::sqlite::insert_log(
        scan.id.clone(),
        "info".to_string(),
        format!("Enumeration tasks spawned: {}", args.tasks.clone()),
        &sqlite_pool,
        args.log_level.to_string()
    )
    .await?;
    // thread responsible for mutating tui data
    threads.push(tokio::spawn(task_handles::result_mutator::handle(
        scan.id.clone(),
        sqlite_pool.clone(),
        results_arc.clone(),
        logs_arc.clone(),
        args.clone(),
    )));

    // setup tui
    let mut terminal = ratatui::init();
    tui::Tui {
        pause: is_paused,
        halt: false,
        home_scroll_offset: 0,
        logs_scroll_offset: 0,
        refresh_rate: 1.0,
        sqlite_pool: sqlite_pool.clone(),
        scan_id: scan.id.clone(),
        results: results_arc,
        status: "Running".to_string(),
        current_tab: tui::Tab::Home,
        logs: logs_arc.clone(),
        args: args.clone(),
        output_written: false,
        log_level: "debug".to_string(),
        method_filter: "none".to_string(),
    }
    .run(&mut terminal)
    .await?;
    // exec all threads
    futures::future::join_all(threads).await;

    Ok(())
}
