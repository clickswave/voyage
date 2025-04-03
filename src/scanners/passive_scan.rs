use std::collections::HashMap;
use crate::libs::args::Args;

pub async fn execute(
    domain: &String,
    args: Args
) -> Result<HashMap<String, String>, anyhow::Error>{

    let user_agent = match args.passive_random_user_agent {
        true => crate::libs::rng::user_agent(),
        false => args.passive_user_agent.clone(),
    };

    let reqwest_client = reqwest::Client::builder().user_agent(user_agent).build()?;

    let mut passive_scan_result = HashMap::new();

    let crt_sh_results = crate::scanners::providers::crt_sh::fetch(
        &reqwest_client,
        domain,
    ).await?;

    for subdomain in crt_sh_results {
        passive_scan_result.insert(
            subdomain.to_string(),
            "crt.sh".to_string(),
        );
    }

    Ok(passive_scan_result)
}