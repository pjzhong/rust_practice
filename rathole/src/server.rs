use crate::Config;
use anyhow::Result;
use tokio::sync::broadcast;

pub async fn run_server(config: &Config, shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    Ok(())
}
