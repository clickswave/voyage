use std::collections::HashSet;
use reqwest::Client;

pub async fn fetch(
    reqwest_client: &Client,
    domain: &str
) -> Result<Vec<String>, anyhow::Error> {
    let mut results = vec![];

    let url = format!("https://api.hackertarget.com/hostsearch/?q={}", domain);
    let response = reqwest_client.get(&url).send().await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let mut unique_subdomains = HashSet::new();

        for line in body.lines() {
            if let Some((subdomain, _ip)) = line.split_once(',') {
                if subdomain.ends_with(domain) {
                    let stripped = subdomain.strip_suffix(domain).unwrap_or("").trim_end_matches('.');
                    if !stripped.is_empty() && stripped != "*" {
                        unique_subdomains.insert(stripped.to_string());
                    }
                    unique_subdomains.insert(stripped.to_string());
                }
            }
        }

        results.extend(unique_subdomains.into_iter());
    }

    Ok(results)
}
