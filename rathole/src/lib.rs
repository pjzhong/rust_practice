mod cli;
mod config_watcher;
mod config;

use anyhow::Result;

use tokio::sync::broadcast;
pub use cli::Cli;

pub async fn run(args: Cli, shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {

    let config_path = args.config_path.as_ref().expect("config path is not exists");

    // config watcher, 热更文件？


    Ok(())
}
