use std::collections::HashSet;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct CrtResponse {
    common_name: String,
}

pub async fn fetch(
    reqwest_client: &Client,
    domain: &String
) -> Result<Vec<String>, anyhow::Error> {
    let mut results = vec![];

    let url = format!("https://crt.sh/?q={}&output=json", domain);
    let response = reqwest_client.get(&url).send().await?;

    if response.status().is_success() {
        let body: Vec<CrtResponse> = response.json().await?;
        let mut unique_subdomains = HashSet::new();
        for entry in body {
            let subdomain = entry.common_name;
            if subdomain.ends_with(domain) {
                let stripped = subdomain.strip_suffix(domain).unwrap_or("").trim_end_matches('.');
                if !stripped.is_empty() {
                    unique_subdomains.insert(stripped.to_string());
                }
            }
        }
        results.extend(unique_subdomains.into_iter());
    }

    Ok(results)
}
