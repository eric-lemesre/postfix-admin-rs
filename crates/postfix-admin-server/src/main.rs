//! Main server binary for postfix-admin-rs.

use postfix_admin_core::config::CliOverrides;
use postfix_admin_core::AppConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = CliOverrides::default();
    let (config, _warnings) = AppConfig::load(&cli)?;

    tracing::info!(
        bind = %config.server.bind_address,
        port = config.server.port,
        "postfix-admin-rs server starting"
    );

    Ok(())
}
