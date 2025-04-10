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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::{self, File};
    use tokio::io::AsyncWriteExt;
    use std::path::Path;

    async fn setup_test_dir() -> Result<String, anyhow::Error> {
        let dir_path = format!("target/test_temp_{}", uuid::Uuid::new_v4());
        fs::create_dir_all(&dir_path).await?;
        Ok(dir_path)
    }

    async fn create_test_file(dir: &str, filename: &str, content: &str) -> Result<String, anyhow::Error> {
        let file_path = Path::new(dir).join(filename);
        let mut file = File::create(&file_path).await?;
        file.write_all(content.as_bytes()).await?;
        file.sync_all().await?;
        Ok(file_path.to_str().unwrap().to_string())
    }

    async fn cleanup_test_dir(dir: &str) -> Result<(), anyhow::Error> {
        fs::remove_dir_all(dir).await?;
        Ok(())
    }


    #[tokio::test]
    async fn test_read_lines_normal_case() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let content = "line one\nline two\n\nline four\n";
        let file_path = create_test_file(&test_dir, "normal.txt", content).await?;
        let result = read_lines(&file_path).await;
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "line one");
        assert_eq!(lines[1], "line two");
        assert_eq!(lines[2], ""); // Empty line
        assert_eq!(lines[3], "line four");
        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_lines_empty_file() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let file_path = create_test_file(&test_dir, "empty.txt", "").await?;
        let result = read_lines(&file_path).await;
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert!(lines.is_empty());
        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_lines_single_line_no_newline() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let file_path = create_test_file(&test_dir, "single_no_nl.txt", "hello world").await?;
        let result = read_lines(&file_path).await;
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "hello world");
        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

     #[tokio::test]
    async fn test_read_lines_single_line_with_newline() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let file_path = create_test_file(&test_dir, "single_with_nl.txt", "hello world\n").await?;
        let result = read_lines(&file_path).await;
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "hello world");
        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_lines_file_not_found() -> Result<(), anyhow::Error> {
        let non_existent_path = format!("target/test_temp_{}/non_existent_file.txt", uuid::Uuid::new_v4());
        let _ = fs::remove_dir_all(Path::new(&non_existent_path).parent().unwrap()).await;
        let result = read_lines(&non_existent_path).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_sha512_known_content() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let content = "This is a test string.\nIt spans multiple lines.\n";
        let expected_hash = "9e32a2085a88c8971e81a3e58e953090932194805ecb8c6f902c27f2c45dc2980ab404cf00670997ff561210d4f509720d7c9a2a23385047a1d59abecdfc6b22";
        let file_path = create_test_file(&test_dir, "known_content.txt", content).await?;

        let result = sha512(&file_path).await;

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash, expected_hash);

        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

     #[tokio::test]
    async fn test_sha512_large_content() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let content = "a".repeat(5000);
        let expected_hash = "0a76c190740222c4ae52b525ff4055e06543c22cd1617f342113742befe4a8ea3e50e1e5491e9bcfcb25baf041734ff1f5675207b6d7fc7f9c334db07e8e48c0";
        let file_path = create_test_file(&test_dir, "large_content.txt", &content).await?;

        let result = sha512(&file_path).await;

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash, expected_hash);

        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sha512_empty_file() -> Result<(), anyhow::Error> {
        let test_dir = setup_test_dir().await?;
        let file_path = create_test_file(&test_dir, "empty_hash.txt", "").await?;

        let expected_hash = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e";

        let result = sha512(&file_path).await;

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert_eq!(hash, expected_hash);

        cleanup_test_dir(&test_dir).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sha512_file_not_found() -> Result<(), anyhow::Error> {
        let non_existent_path = format!("target/test_temp_{}/non_existent_file_hash.txt", uuid::Uuid::new_v4());
        let _ = fs::remove_dir_all(Path::new(&non_existent_path).parent().unwrap()).await;

        let result = sha512(&non_existent_path).await;
        assert!(result.is_err());

        Ok(())
    }
}
