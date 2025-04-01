use serde::Serialize;
use crate::{libs};
use sqlx::Error::Database;
use sqlx::{Executor, FromRow, Pool, Sqlite};
use tokio::fs;

pub async fn init(db_path: String) -> Result<Pool<Sqlite>, sqlx::Error> {
    // create db if not exists
    let db_exists = std::path::Path::new(db_path.as_str()).exists();
    if !db_exists {
        fs::write(&db_path, b"").await?
    }
    let sqlite_pool = sqlx::SqlitePool::connect(format!("sqlite:{db_path}").as_str()).await?;
    // Enable WAL mode
    sqlite_pool.execute("PRAGMA journal_mode=WAL;").await?;
    // sync normal
    sqlite_pool.execute("PRAGMA synchronous=NORMAL;").await?;
    // create table scans if not exists

    // create table scans if not exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scans (
            id TEXT NOT NULL PRIMARY KEY,
            config_hash TEXT NOT NULL,
            config TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            no_banner BOOLEAN NOT NULL,
            launch_delay INTEGER NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP),
            updated_at TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP),
            notifications TEXT NOT NULL DEFAULT '{}'
        )
        "#
    ).execute(&sqlite_pool).await?;

    // create table logs if not exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_id TEXT NOT NULL,
            level TEXT NOT NULL DEFAULT 'debug',
            description TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP)
        )
        "#
    ).execute(&sqlite_pool).await?;

    Ok(sqlite_pool)
}

pub async fn reset_halted_scans(
    scan_id: String,
    sqlite_pool: Pool<Sqlite>,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        format!(
            "
            UPDATE {scan_id}
            SET status = 'queued'
            WHERE status = 'scanning'
            ",
        )
        .as_str(),
    )
    .execute(&sqlite_pool)
    .await?;
    Ok(())
}


#[derive(Serialize, Debug, Clone, FromRow)]
pub struct ScanResult {
    pub domain: String,
    pub subdomain: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct ScanResults {
    pub found: Vec<ScanResult>,
    pub not_found: i32,
    pub total: i32,
}

pub async fn get_results(
    scan_id: String,
    sqlite_pool: Pool<Sqlite>,
) -> std::result::Result<ScanResults, anyhow::Error> {

    let found_results = sqlx::query_as::<_, ScanResult>(
        format!(
            "SELECT subdomain,domain FROM {scan_id} WHERE status = 'found'",
            scan_id = scan_id
        )
        .as_str(),
    ).bind(&scan_id).fetch_all(&sqlite_pool).await?;

    let not_found_count: (i32,) = sqlx::query_as(
        format!(
            "SELECT COUNT(*) FROM {scan_id} WHERE status = 'not found'",
            scan_id = scan_id
        )
        .as_str(),
    )
    .bind(&scan_id)
    .fetch_one(&sqlite_pool)
    .await?;

    let total_count: (i32,) =
        sqlx::query_as(format!("SELECT COUNT(*) FROM {scan_id}", scan_id = scan_id).as_str())
            .bind(&scan_id)
            .fetch_one(&sqlite_pool)
            .await?;

    let results = ScanResults {
        found: found_results,
        not_found: not_found_count.0,
        total: total_count.0,
    };
    Ok(results)

}

#[derive(Debug, Clone, FromRow)]
pub struct Log {
    pub description: String,
    pub level: String,
    pub created_at: String,
}

pub async fn insert_log(
    scan_id: String,
    level: String,
    description: String,
    sqlite_pool: &Pool<Sqlite>,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        format!(
            "INSERT INTO logs (scan_id, level, description) VALUES ('{scan_id}', '{level}', '{description}')",
            scan_id = scan_id,
            level = level,
            description = description
        )
        .as_str(),
    )
    .execute(sqlite_pool)
    .await?;
    Ok(())
}

pub async fn get_logs(
    scan_id: String,
    log_level: String,
    sqlite_pool: Pool<Sqlite>,
) -> Result<Vec<Log>, anyhow::Error> {
    let query_string = match log_level.as_str() {
        "debug" => "SELECT description, level, created_at FROM logs WHERE scan_id = ? AND (level IN ('debug', 'info', 'warn', 'error')) ORDER BY created_at DESC".to_string(),
        "info" => "SELECT description, level, created_at FROM logs WHERE scan_id = ? AND (level IN ('info', 'warn', 'error')) ORDER BY created_at DESC".to_string(),
        "warn" => "SELECT description, level, created_at FROM logs WHERE scan_id = ? AND (level IN ('warn', 'error')) ORDER BY created_at DESC".to_string(),
        "error" => "SELECT description, level, created_at FROM logs WHERE scan_id = ? AND level = 'error' ORDER BY created_at DESC".to_string(),
        _ => "SELECT description, level, created_at FROM logs WHERE scan_id = ? ORDER BY created_at DESC".to_string(),
    };

    let logs = sqlx::query_as::<_, Log>(&query_string)
        .bind(scan_id)
        .fetch_all(&sqlite_pool)
        .await?;

    Ok(logs)
}

pub async fn create_workload_table(
    scan_id: String,
    sqlite_pool: Pool<Sqlite>,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        format!(
            "CREATE TABLE IF NOT EXISTS {scan_id} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subdomain TEXT NOT NULL,
                domain TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT  'queued',
                created_on TIMESTAMP NOT NULL DEFAULT (CURRENT_TIMESTAMP),
                last_scanned_on TIMESTAMP DEFAULT NULL,
                last_scan_started_on TIMESTAMP DEFAULT NULL,
                max_retries INTEGER NOT NULL DEFAULT 0
            )",
        )
        .as_str(),
    )
    .execute(&sqlite_pool)
    .await?;
    // Update scan status
    sqlx::query(
        format!("UPDATE scans SET status = 'workload_table_created' WHERE id = '{scan_id}'").as_str(),
    )
    .execute(&sqlite_pool)
    .await?;
    Ok(())
}

pub async fn populate_basic_workload(
    scan_id: String,
    domain: String,
    wordlist_path: String,
    sqlite_pool: Pool<Sqlite>,
) -> Result<(), String> {
    // Read words from wordlist
    let read_wordlist = match libs::wordlist::read_lines(wordlist_path.as_str()).await {
        Ok(wordlist) => wordlist,
        Err(e) => return Err(e.to_string()),
    };

    // Do not allow proceeding with an empty wordlist
    if read_wordlist.is_empty() {
        return Err("Wordlist is empty".to_string());
    }

    const SQLITE_MAX_VARIABLES: usize = 900; // Set a safe batch size

    let mut chunk_iter = read_wordlist.chunks(SQLITE_MAX_VARIABLES / 2); // (2 bindings per row)

    while let Some(chunk) = chunk_iter.next() {
        let mut query = format!("INSERT INTO \"{}\" (domain, subdomain) VALUES ", scan_id);
        let placeholders: Vec<String> = chunk.iter().map(|_| "(?, ?)".to_string()).collect();
        query.push_str(&placeholders.join(", "));

        let mut sql_query = sqlx::query(&query);
        for subdomain in chunk {
            sql_query = sql_query.bind(domain.as_str()).bind(subdomain);
        }

        // Execute the query
        if let Err(e) = sql_query.execute(&sqlite_pool).await {
            if let Database(ref db_error) = e {
                if !db_error.code().map_or(false, |code| code == "1555") {
                    return Err(format!("Failed to execute multi-insert query: {e}"));
                }
            } else {
                return Err(format!("Failed to execute multi-insert query: {e}"));
            }
        }
    }

    // Change scan status to 'basic_workload_populated'
    let change_status = sqlx::query(
        "UPDATE scans
     SET status = 'basic_workload_populated'
     WHERE id = ?"
    )
        .bind(scan_id)
        .execute(&sqlite_pool)
        .await;

    match change_status {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

