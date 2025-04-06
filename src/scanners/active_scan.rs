use hickory_resolver::TokioResolver;
use reqwest::Client;
use futures::future::join_all;

pub struct NegativeResult {
    pub level: String,
    pub description: String,
}

pub struct ActiveScanResult {
    pub found: bool,
    pub source: String,
    pub negatives: Vec<NegativeResult>,
}

async fn perform_dns_lookup(resolver: &TokioResolver, domain: &str) -> Vec<NegativeResult> {
    match resolver.lookup_ip(domain).await {
        Ok(lookup) => {
            let has_ipv4 = lookup.iter().any(|ip| ip.is_ipv4());
            let has_ipv6 = lookup.iter().any(|ip| ip.is_ipv6());

            let mut negatives = Vec::new();
            if !has_ipv4 {
                negatives.push(NegativeResult {
                    level: "info".into(),
                    description: format!("No IPv4 addresses found for {}", domain),
                });
            }
            if !has_ipv6 {
                negatives.push(NegativeResult {
                    level: "info".into(),
                    description: format!("No IPv6 addresses found for {}", domain),
                });
            }

            negatives
        }
        Err(e) if e.is_no_records_found() => vec![NegativeResult {
            level: "info".into(),
            description: format!("No DNS records found for {}", domain),
        }],
        Err(e) => vec![NegativeResult {
            level: "error".into(),
            description: format!("DNS lookup error for {}: {}", domain, e),
        }],
    }
}

async fn perform_request(client: &Client, protocol: &str, domain: &str) -> Result<(), NegativeResult> {
    let url = format!("{}://{}", protocol, domain);
    match client.get(&url).send().await {
        Ok(_) => Ok(()),
        Err(e) if e.is_timeout() => Err(NegativeResult {
            level: "warn".into(),
            description: format!("{} request timed out for {}", protocol.to_uppercase(), domain),
        }),
        Err(e) => Err(NegativeResult {
            level: "error".into(),
            description: format!("{} request error: {}", protocol.to_uppercase(), e),
        }),
    }
}

pub async fn execute(
    resolver: &TokioResolver,
    reqwest_client: &Client,
    domain: &str,
) -> ActiveScanResult {
    let protocols = ["http", "https", "ftp", "smtp"];
    let protocol_futures = protocols
        .iter()
        .map(|&proto| perform_request(reqwest_client, proto, domain));

    // Run DNS and protocol checks concurrently
    let (dns_negatives, protocol_results) = tokio::join!(
        perform_dns_lookup(resolver, domain),
        join_all(protocol_futures)
    );

    let mut scan_result = ActiveScanResult {
        found: false,
        source: String::new(),
        negatives: dns_negatives,
    };

    if scan_result.negatives.iter().all(|n| n.level != "error") {
        scan_result.found = true;
        return scan_result;
    }

    for result in protocol_results {
        match result {
            Ok(_) => {
                scan_result.found = true;
                return scan_result;
            }
            Err(neg) => scan_result.negatives.push(neg),
        }
    }

    scan_result
}
