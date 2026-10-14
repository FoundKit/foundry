pub mod hooks;
pub mod systems;

use foundry::prelude::*;
use hooks::BlogMutationHook;
use systems::{BlogSubsystem, NewsletterSubsystem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build configuration from environment variables
    let config = FoundryConfig::from_env();

    // 2. Initialize and configure the Foundry application
    let app = FoundryApp::builder()
        .config(config)
        .register_subsystem(BlogSubsystem)
        .register_subsystem(NewsletterSubsystem)
        .register_hook(BlogMutationHook)
        .build()
        .await?;

    // 3. Start serving traffic
    app.run().await?;
    Ok(())
}
