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

    let domain_string = domain.to_string();
    let crt_sh_results = crate::scanners::providers::crt_sh::fetch(&client, &domain_string).await?;

    let mut passive_scan_result = HashMap::new();

    passive_scan_result.extend(
        crt_sh_results
            .into_iter()
            .map(|subdomain| (subdomain.to_string(), "crt.sh".to_string()))
            .collect::<HashMap<String, String>>(),
    );

    Ok(passive_scan_result)
}
