use hickory_resolver::TokioResolver;
use reqwest::Client;


pub struct NegativeResult {
    pub level: String,
    pub description: String,
}
pub struct ActiveScanResult {
    pub found: bool,
    pub source: String,
    pub negatives: Vec<NegativeResult>,
}

pub async fn execute(resolver: &TokioResolver, reqwest_client: &Client, domain: &String) -> ActiveScanResult {
    let mut scan_result = ActiveScanResult {
        found: false,
        source: "".to_string(),
        negatives: vec![],
    };

    let ipv4_lookup = resolver.ipv4_lookup(domain).await;
    match ipv4_lookup {
        Ok(_) => {
            scan_result.found = true;
            return scan_result;
        },
        Err(e) => {
            match e.is_no_records_found() {
                true => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "info".to_string(),
                            description: format!("No IPv4 addresses found for {}", domain),
                        }
                    )
                }
                false => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "error".to_string(),
                            description: format!("IPv4 lookup error: {}", e),
                        }
                    )
                }
            }
        }
    }

    let ipv6_lookup = resolver.ipv6_lookup(domain).await;
    match ipv6_lookup {
        Ok(_) => {
            scan_result.found = true;
            return scan_result;
        },
        Err(e) => {
            match e.is_no_records_found() {
                true => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "info".to_string(),
                            description: format!("No IPv6 addresses found for {}", domain),
                        }
                    );
                }
                false => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "error".to_string(),
                            description: format!("IPv6 lookup error: {}", e),
                        }
                    );
                }
            }
        }
    }

    let request = reqwest_client.get(format!("http://{domain}")).send().await;
    match request {
        Ok(_) => {
            scan_result.found = true;
            return scan_result;
        },
        Err(e) => {
            match e.is_timeout() {
                true => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "warn".to_string(),
                            description: format!("HTTP request timed out for {}", domain),
                        }
                    );
                }
                false => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "error".to_string(),
                            description: format!("HTTP request error: {}", e),
                        }
                    );
                }
            }
        }
    }

    let request = reqwest_client.get(format!("https://{domain}")).send().await;
    match request {
        Ok(_) => {
            scan_result.found = true;
            return scan_result;
        },
        Err(e) => {
            match e.is_timeout() {
                true => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "warn".to_string(),
                            description: format!("HTTPS request timed out for {}", domain),
                        }
                    );
                }
                false => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "error".to_string(),
                            description: format!("HTTPS request error: {}", e),
                        }
                    );
                }
            }
        }
    }

    let request = reqwest_client.get(format!("ftp://{domain}")).send().await;
    match request {
        Ok(_) => {
            scan_result.found = true;
            return scan_result;
        },
        Err(e) => {
            match e.is_timeout() {
                true => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "warn".to_string(),
                            description: format!("FTP request timed out for {}", domain),
                        }
                    );
                }
                false => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "error".to_string(),
                            description: format!("FTP request error: {}", e),
                        }
                    );
                }
            }
        }
    }

    let request = reqwest_client.get(format!("smtp://{domain}")).send().await;
    match request {
        Ok(_) => {
            scan_result.found = true;
            return scan_result;
        },
        Err(e) => {
            match e.is_timeout() {
                true => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "warn".to_string(),
                            description: format!("SMTP request timed out for {}", domain),
                        }
                    );
                }
                false => {
                    scan_result.negatives.push(
                        NegativeResult {
                            level: "error".to_string(),
                            description: format!("SMTP request error: {}", e),
                        }
                    );
                }
            }
        }
    }

    scan_result
}