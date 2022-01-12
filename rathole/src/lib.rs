use anyhow::Result;
use tokio::sync::broadcast;

pub use cli::Cli;

use crate::client::run_client;
use crate::config::Config;
use crate::server::run_server;

mod cli;
mod client;
mod config;
mod helper;
mod protocol;
mod server;
mod transport;

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
    let ret: Result<()> = match determine_run_mode(&config, &arg) {
        RunMode::Server => {
            #[cfg(not(feature = "server"))]
            helper::feature_not_compile("server");

            #[cfg(feature = "server")]
            run_server(&config, shutdown_rx).await
        }
        RunMode::Client => {
            #[cfg(not(feature = "client"))]
            helper::feature_not_compile("client");

            #[cfg(feature = "client")]
            run_client(&config, shutdown_rx).await
        }
        RunMode::Undetermined => panic!("Cannot determine running as a server or a client"),
    };

    ret.unwrap();
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
