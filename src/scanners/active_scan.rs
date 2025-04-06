use crate::scanners::techniques;
use hickory_resolver::TokioResolver;
use reqwest::Client;
use crate::libs::args::Args;

pub struct NegativeResult {
    pub level: String,
    pub description: String,
}

pub struct ActiveScanResult {
    pub found: bool,
    pub source: String,
    pub negatives: Vec<NegativeResult>,
}

pub async fn execute(
    resolver: &TokioResolver,
    reqwest_client: &Client,
    args: &Args,
    domain: &String,
) -> ActiveScanResult {
    let mut scan_result = ActiveScanResult {
        found: false,
        source: "".to_string(),
        negatives: vec![],
    };
    // ipv4 lookup
    if !args.exclude_active_technique.contains(&"ipv4_lookup".to_string()) {
        let ipv4_lookup = techniques::ipv4_lookup::execute(resolver, domain).await;
        match ipv4_lookup {
            Ok(_) => {
                scan_result.found = true;
                scan_result.source = "ipv4_lookup".to_string();
                return scan_result;
            }
            Err(e) => scan_result.negatives.push(e),
        }
    }
    // ipv6 lookup
    if !args.exclude_active_technique.contains(&"ipv6_lookup".to_string()) {
        let ipv6_lookup = techniques::ipv6_lookup::execute(resolver, domain).await;
        match ipv6_lookup {
            Ok(_) => {
                scan_result.found = true;
                scan_result.source = "ipv6_lookup".to_string();
                return scan_result;
            }
            Err(e) => scan_result.negatives.push(e),
        }
    }
    // http probing
    if !args.exclude_active_technique.contains(&"http_probing".to_string()) {
        let http_probing = techniques::http_probing::execute(reqwest_client, domain, &args.http_probing_port).await;
        match http_probing {
            Ok(_) => {
                scan_result.found = true;
                scan_result.source = "http_probing".to_string();
                return scan_result;
            }
            Err(e) => scan_result.negatives.extend(e),
        }
    }
    // https probing
    if !args.exclude_active_technique.contains(&"https_probing".to_string()) {
        let https_probing = techniques::https_probing::execute(reqwest_client, domain, &args.https_probing_port).await;
        match https_probing {
            Ok(_) => {
                scan_result.found = true;
                scan_result.source = "https_probing".to_string();
                return scan_result;
            }
            Err(e) => scan_result.negatives.extend(e),
        }
    }
    scan_result
}
