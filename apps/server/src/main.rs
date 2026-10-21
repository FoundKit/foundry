use foundry::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = FoundryConfig::from_env();
    let app = match FoundryApp::builder().config(config).build().await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("\n❌ Application startup failed: {}\n", e);
            std::process::exit(1);
        }
    };

    app.run().await?;
    Ok(())
}
