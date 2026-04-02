//! Main server binary for postfix-admin-rs.

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("postfix-admin-rs server v0.1.0");

    Ok(())
}
