use std::collections::HashMap;
use crate::libs::args::Args;

pub async fn execute(domain: &str, args: Args) -> Result<HashMap<String, String>, anyhow::Error> {
    let user_agent = if args.passive_random_user_agent {
        crate::libs::rng::user_agent()
    } else {
        args.passive_user_agent.clone()
    };

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;

    let mut passive_scan_result = HashMap::new();
    // crt.sh
    if !args.exclude_passive_source.contains(&"crt.sh".to_string()) {
        let crt_sh_results = crate::scanners::providers::crt_sh::fetch(&client, domain).await?;
        passive_scan_result.extend(
            crt_sh_results
                .into_iter()
                .map(|subdomain| (subdomain.to_string(), "crt.sh".to_string()))
                .collect::<HashMap<String, String>>(),
        );
    }
    // hackertarget
    if !args.exclude_passive_source.contains(&"hackertarget".to_string()) {
        let hackertarget = crate::scanners::providers::hackertarget::fetch(&client, domain).await?;
        passive_scan_result.extend(
            hackertarget
                .into_iter()
                .map(|subdomain| (subdomain.to_string(), "hackertarget".to_string()))
                .collect::<HashMap<String, String>>(),
        );
    }
    // alienvault
    if !args.exclude_passive_source.contains(&"alienvault".to_string()) {
        let alienvault = crate::scanners::providers::alienvault::fetch(&client, domain).await?;
        passive_scan_result.extend(
            alienvault
                .into_iter()
                .map(|subdomain| (subdomain.to_string(), "alienvault".to_string()))
                .collect::<HashMap<String, String>>(),
        );
    }

    Ok(passive_scan_result)
}
