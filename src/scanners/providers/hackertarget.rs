use std::collections::HashSet;
use reqwest::Client;

pub async fn fetch(
    reqwest_client: &Client,
    domain: &str
) -> Result<Vec<String>, anyhow::Error> {
    let url = format!("https://api.hackertarget.com/hostsearch/?q={}", domain);
    let response = reqwest_client.get(&url).send().await?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    let body = response.text().await?;
    let mut unique_subdomains = HashSet::new();

    for line in body.lines() {
        if let Some((subdomain, _)) = line.split_once(',') {
            if subdomain.ends_with(domain) {
                let cleaned = subdomain.trim_end_matches('.');

                // Ensure it's not the base domain and not a wildcard
                if cleaned != domain && !cleaned.starts_with("*.") {
                    unique_subdomains.insert(cleaned.to_string());
                }
            }
        }
    }

    Ok(unique_subdomains.into_iter().collect())
}
