use sha2::Digest;

// generate sha512 from a json string
pub async fn sha512(string: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut hasher = sha2::Sha512::new();
    hasher.update(string);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}