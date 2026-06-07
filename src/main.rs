mod core;
mod infrastructure;
mod presentation;

use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("enterprise_web=debug,tower_http=debug")
        .init();

    let config = Config::from_file_and_env()?;
    tracing::info!("Loaded config: {:?}", config);

    presentation::run(config).await?;
    Ok(())
}