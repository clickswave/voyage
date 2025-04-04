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
    let lookups = vec![
        ("IPv4", resolver.ipv4_lookup(domain)),
        ("IPv6", resolver.ipv6_lookup(domain)),
    ];

    let results = join_all(lookups.into_iter().map(|(proto, lookup)| async move {
        match lookup.await {
            Ok(_) => None,
            Err(e) if e.is_no_records_found() => Some(NegativeResult {
                level: "info".into(),
                description: format!("No {} addresses found for {}", proto, domain),
            }),
            Err(e) => Some(NegativeResult {
                level: "error".into(),
                description: format!("{} lookup error: {}", proto, e),
            }),
        }
    })).await;

    results.into_iter().filter_map(|res| res).collect()
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
    let mut scan_result = ActiveScanResult {
        found: false,
        source: "".into(),
        negatives: vec![],
    };

    // DNS Checks concurrently
    scan_result.negatives.extend(perform_dns_lookup(resolver, domain).await);

    if scan_result.negatives.iter().all(|n| n.level != "error") {
        scan_result.found = true;
        return scan_result;
    }

    // Protocol checks concurrently
    let protocols = vec!["http", "https", "ftp", "smtp"];
    let protocol_checks = protocols.into_iter().map(|proto| perform_request(reqwest_client, proto, domain));
    let protocol_results = join_all(protocol_checks).await;

    for result in protocol_results {
        match result {
            Ok(_) => {
                scan_result.found = true;
                return scan_result; // Early return if service found
            }
            Err(negative) => scan_result.negatives.push(negative),
        }
    }

    scan_result
}
