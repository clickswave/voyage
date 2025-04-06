use std::collections::HashSet;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct Response {
    passive_dns: Vec<PassiveDnsRecord>,
}

#[derive(Deserialize)]
struct PassiveDnsRecord {
    hostname: String,
}

pub async fn fetch(
    reqwest_client: &Client,
    domain: &str
) -> Result<Vec<String>, anyhow::Error> {
    let mut results = vec![];
    let url = format!(
        "https://otx.alienvault.com/api/v1/indicators/domain/{}/passive_dns",
        domain
    );

    let response = reqwest_client.get(&url).send().await?;

    if response.status().is_success() {
        let data: Response = response.json().await?;
        let mut unique_subdomains = HashSet::new();

        for record in data.passive_dns {
            let hostname = record.hostname;
            if hostname.ends_with(domain) && hostname != *domain {
                unique_subdomains.insert(hostname);
            }
        }

        results.extend(unique_subdomains);
    }

    Ok(results)
}
