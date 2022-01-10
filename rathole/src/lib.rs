mod cli;
mod config;
mod server;

use anyhow::Result;
use core::panicking::panic;

use crate::config::Config;
pub use cli::Cli;
use tokio::sync::broadcast;

#[derive(PartialEq, Eq, Debug)]
enum RunMode {
    Server,
    Client,
    Undetermined,
}

pub async fn run(args: Cli, shutdown_tx: broadcast::Sender<bool>) -> Result<()> {
    let config_path = args
        .config_path
        .as_ref()
        .expect("config path is not exists");

    let config = Config::from_file(config_path)?;

    let join = tokio::spawn(run_instance(
        config.clone(),
        args.clone(),
        shutdown_tx.subscribe(),
    ));

    join.await?;
    shutdown_tx.send(true);

    Ok(())
}

async fn run_instance(config: Config, arg: Cli, shutdown_rx: broadcast::Receiver<bool>) {
    let ret: Result<()> = match determine_run_mode(&config, &cli) {
        RunMode::Server =>
        {
            #[cfg(feature = "client")]
            run_server().await
        }
        RunMode::Client =>
        {
            #[cfg(feature = "server")]
            run_client().await
        }
        RunMode::Undetermined => panic!("Cannot determine running as a server or a client"),
    };

    ret.unwrap();
}

pub async fn run_client() -> Result<()> {
    Ok(())
}

pub async fn run_server(config: &Config, args: &Cli) -> Result<()> {
    Ok(())
}

fn determine_run_mode(config: &Config, args: &Cli) -> RunMode {
    use RunMode::*;
    if args.client && args.server {
        Undetermined
    } else if args.client {
        Client
    } else if args.server {
        Server
    } else if config.client.is_some() && config.server.is_none() {
        Client
    } else if config.client.is_none() && config.server.is_some() {
        Server
    } else {
        Undetermined
    }
}
