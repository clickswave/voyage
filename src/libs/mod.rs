use hickory_resolver::{TokioResolver};
use reqwest::Client;
pub (crate) mod args;
pub (crate) mod banner;
pub (crate) mod wordlist;
pub (crate) mod sqlite;
pub (crate) mod rng;
pub (crate) mod sha;
pub (crate) mod dns;

pub async fn domain_exists (resolver: &TokioResolver, reqwest_client: &Client, domain: &String) -> Result<bool, Vec<String>> {


    let mut errors = vec![];

    let ipv4_lookup = resolver.ipv4_lookup(domain).await;
    match ipv4_lookup {
        Ok(_) => return Ok(true),
        Err(e) => {
            errors.push(format!("IPv4 lookup error: {}", e));
        }
    }

    let ipv6_lookup = resolver.ipv6_lookup(domain).await;
    match ipv6_lookup {
        Ok(_) => return Ok(true),
        Err(e) => {
            errors.push(format!("IPv6 lookup error: {}", e));
        }
    }

    let request = reqwest_client.get(format!("http://{domain}")).send().await;
    match request {
        Ok(_) => return Ok(true),
        Err(e) => {
            errors.push(format!("HTTP request error: {}", e));
        }
    }

    let request = reqwest_client.get(format!("https://{domain}")).send().await;
    match request {
        Ok(_) => return Ok(true),
        Err(e) => {
            errors.push(format!("HTTPS request error: {}", e));
        }
    }

    if errors.is_empty() {
        Ok(false)
    } else {
        Err(errors)
    }
}
