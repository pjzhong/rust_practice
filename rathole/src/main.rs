use anyhow::Result;
use clap::Parser;
use rathole::{run, Cli};
use tokio::signal;
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let (shutdown_tx, _) = broadcast::channel::<bool>(1);
    let shutdown_tx_ctrl_c = shutdown_tx.clone();
    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            // Something really weired happened. So just panic
            panic!("Failed to listen for the ctrl-c signal: {:?}", e);
        }

        if let Err(e) = shutdown_tx_ctrl_c.send(true) {
            // shutdown signal must be catch and handle properly
            panic!("Failed to send shutdown signal: {:?}", e);
        }
    });

    let level = "info"; // if RUST_LOG not present, use `info` level
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::from(level)),
        )
        .init();

    run(args, shutdown_tx).await
}
