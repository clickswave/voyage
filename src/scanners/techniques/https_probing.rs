use crate::scanners::active_scan::NegativeResult;

pub async fn execute(
    reqwest_client: &reqwest::Client,
    domain: &String,
    ports: &Vec<u16>,
) -> Result<(), Vec<NegativeResult>> {
    let mut negatives = vec![];
    for port in ports {
        let request = reqwest_client.get(format!("https://{domain}:{port}")).send().await;
        match request {
            Ok(_) => return Ok(()),
            Err(e) => match e.is_timeout() {
                true => negatives.push(NegativeResult {
                    level: "info".to_string(),
                    description: format!("HTTPS request timeout for {domain}:{port} {e}"),
                }),
                false => negatives.push(NegativeResult {
                    level: "info".to_string(),
                    description: format!("HTTPS request failed for {domain}:{port} {e}"),
                }),
            },
        }
    }
    Err(negatives)
}
