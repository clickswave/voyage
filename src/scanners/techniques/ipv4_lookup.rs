use crate::scanners::active_scan::NegativeResult;

#[allow(dead_code)]
pub async fn execute(
    resolver: &hickory_resolver::TokioResolver,
    domain: &String,
) -> Result<(), NegativeResult> {
    let ipv4_lookup = resolver.ipv4_lookup(domain).await;
    match ipv4_lookup {
        Ok(_) => Ok(()),
        Err(e) => match e.is_no_records_found() {
            true => Err(NegativeResult {
                level: "info".to_string(),
                description: format!("No IPv4 addresses found for {}", domain),
            }),
            false => Err(NegativeResult {
                level: "error".to_string(),
                description: format!("IPv4 lookup error: {}", e),
            }),
        },
    }
}
