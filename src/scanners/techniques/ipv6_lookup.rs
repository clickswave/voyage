use crate::scanners::active_scan::NegativeResult;

#[allow(dead_code)]
pub async fn execute(
    resolver: &hickory_resolver::TokioResolver,
    domain: &String,
) -> Result<(), NegativeResult> {
    let ipv6_lookup = resolver.ipv6_lookup(domain).await;
    match ipv6_lookup {
        Ok(_) => Ok(()),
        Err(e) => match e.is_no_records_found() {
            true => Err(NegativeResult {
                level: "info".to_string(),
                description: format!("No IPv6 addresses found for {}", domain),
            }),
            false => Err(NegativeResult {
                level: "error".to_string(),
                description: format!("IPv6 lookup error: {}", e),
            }),
        },
    }
}
