use sqlx::Pool;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::libs::sqlite::{get_results};

pub async fn csv (
    scan_id: String,
    sqlite_pool: Pool<sqlx::Sqlite>,
    file_path: String,
) -> Result<(), anyhow::Error>{
    let get_results = get_results(
        scan_id.clone(),
        sqlite_pool.clone()
    ).await?;

    let mut wtr = csv::Writer::from_path(file_path)?;
    for result in get_results.found {
        wtr.serialize(result)?;
    }

    wtr.flush()?;

    Ok(())
}

pub async fn text(
    scan_id: String,
    sqlite_pool: Pool<sqlx::Sqlite>,
    file_path: String,
) -> Result<(), anyhow::Error>{
    let get_results = get_results(
        scan_id.clone(),
        sqlite_pool.clone()
    ).await?;

    let mut file = File::create(file_path).await?;
    for result in get_results.found {
        let line = format!("{}.{}\n", result.subdomain, result.domain);
        file.write_all(line.as_bytes()).await?;
    }

    Ok(())
}

pub async fn export(
    scan_id: String,
    sqlite_pool: Pool<sqlx::Sqlite>,
    file_path: String,
    output_format: String,
) -> Result<(), anyhow::Error> {
    match output_format.as_str() {
        "csv" => csv(scan_id, sqlite_pool, file_path).await,
        "text" => text(scan_id, sqlite_pool, file_path).await,
        _ => Err(anyhow::anyhow!("Invalid output format")),
    }
}
