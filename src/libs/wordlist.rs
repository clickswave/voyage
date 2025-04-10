use sha2::Digest;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

pub async fn read_lines(file_path: &str) -> Result<Vec<String>, anyhow::Error> {
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut result = Vec::new();

    while let Some(line) = lines.next_line().await? {
        result.push(line);
    }

    Ok(result)
}

// get sha512 hash of a file
pub async fn sha512(file_path: &str) -> Result<String, anyhow::Error> {
    let mut file = File::open(file_path).await?;
    let mut hasher = sha2::Sha512::new();
    let mut buffer = [0; 4096];
    while let Ok(n) = file.read(&mut buffer).await {
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let hash_result = hasher.finalize();
    Ok(format!("{:x}", hash_result))
}
