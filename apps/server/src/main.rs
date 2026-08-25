use foundry::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = FoundryConfig::from_env();
    let app = FoundryApp::builder().config(config).build().await?;

    app.run().await?;
    Ok(())
}
