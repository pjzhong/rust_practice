use clap::Parser;
use rathole::Cli;
use std::path::PathBuf;
use tokio::signal;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let (shutdown_tx, shutdown_rx) = broadcast::channel::<bool>(1);
    tokio::spawn(async move {
       if let Err(e) = signal::ctrl_c().await {
           // Something really weired happened. So just panic
           panic!("Failed to listen for the ctrl-c signal: {:?}", e);
       }

        if let Err(e) = shutdown_tx.send(true) {
            // shutdown signal must be catch and handle properly
            panic!("Failed to send shutdown signal: {:?}", e);
        }
    });


    while true {
        println!("Hello, world!");
    }

}
