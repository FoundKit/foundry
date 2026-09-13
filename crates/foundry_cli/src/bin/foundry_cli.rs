#[tokio::main]
async fn main() -> anyhow::Result<()> {
    foundry_cli::run_cli().await
}
