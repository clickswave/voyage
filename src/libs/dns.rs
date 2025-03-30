use hickory_resolver::{Resolver, TokioResolver};

pub fn create_resolver() -> Result<TokioResolver, anyhow::Error> {
    Ok(Resolver::builder_tokio()?.build())
}
