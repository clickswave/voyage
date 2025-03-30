use sha2::Digest;
use tokio::fs::{read_to_string, File};
use tokio::io::{AsyncReadExt};

pub async fn read_lines(file_path: &str) -> Result<Vec<String>, anyhow::Error> {
    let mut result = Vec::new();

    for line in read_to_string(file_path).await?.lines() {
        result.push(line.to_string())
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
