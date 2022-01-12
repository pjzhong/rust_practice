use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use tokio::io::{self};
use tokio::sync::{broadcast, mpsc};
use tokio::time;

use crate::{Config, protocol};
use crate::config::{ServerConfig, ServerServiceConfig, TransportType};
use crate::protocol::{HASH_WIDTH_IN_BYTES, Hello, read_hello};
use crate::transport::{TcpTransport, Transport};

type ServiceDigest = protocol::Digest;

pub async fn run_server(config: &Config, shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    let config = match &config.server {
        Some(config) => config,
        None => {
            return Err(anyhow!("Try to run as a server, but the configuration is missing. Please add the `[server]` block"));
        }
    };

    match config.transport {
        TransportType::Tcp => {
            let mut server = Server::<TcpTransport>::from(config).await?;
            server.run(shutdown_rx).await?;
        }
    }

    Ok(())
}

pub struct ControlChannelHandle<T: Transport> {
    shutdown_tx: broadcast::Sender<bool>,
    data_channel_tx: mpsc::Sender<T::Stream>,
}

struct Server<'a, T: Transport> {
    // `[server]` config
    config: &'a ServerConfig,
    // `[server.services]` config, indexed by ServiceDigest
    services: Arc<RwLock<HashMap<ServiceDigest, ServerServiceConfig>>>,
    // Collection of control channels
    control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
    // Wrapper around the transport layer
    transport: Arc<T>,
}

fn generate_service_hashmap(
    server_config: &ServerConfig,
) -> HashMap<ServiceDigest, ServerServiceConfig> {
    let mut ret = HashMap::new();
    for (key, config) in &server_config.services {
        ret.insert(protocol::digest(key.as_bytes()), config.clone());
    }
    ret
}

impl<'a, T: Transport> Server<'a, T> {
    pub async fn from(config: &'a ServerConfig) -> Result<Server<'a, T>> {
        Ok(Server {
            config,
            services: Arc::new(RwLock::new(generate_service_hashmap(config))),
            control_channels: Arc::new(RwLock::new(HashMap::new())),
            transport: Arc::new(T::new(&config.transport).await?),
        })
    }

    pub async fn run(&mut self, mut shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
        let l = self
            .transport
            .bind(&self.config.bind_addr)
            .await
            .with_context(|| "Failed to listen at `server.bind_addr`")?;

        println!("Listening at {}", self.config.bind_addr);

        let mut backoff = ExponentialBackoff {
            max_interval: Duration::from_millis(100),
            max_elapsed_time: None,
            ..Default::default()
        };

        loop {
            tokio::select! {
                ret = self.transport.accept(&l) => {
                    match ret {
                        Err(err) => {
                            if let Some(err) = err.downcast_ref::<io::Error>() {
                                if let Some(d) = backoff.next_backoff() {
                                    time::sleep(d).await;
                                } else {
                                    println!("Too man retires. Aborting...");
                                    break;
                                }
                            }
                        }
                        Ok((conn, addr)) => {
                            backoff.reset();
                            println!("Incoming connection from {}", addr);

                            let services = self.services.clone();
                            let control_channels = self.control_channels.clone();
                            tokio::spawn(async move {
                                if let Err(err) = handle
                            });
                        }
                    }
                },
                _ = shutdown_rx.recv() => {
                    println!("Shutting down gracefully...");
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection<T: 'static + Transport>(
    mut conn: T::Stream,
    addr: SocketAddr,
    services: Arc<RwLock<HashMap<ServiceDigest, ServerServiceConfig>>>,
    control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
) -> Result<()> {

    let hello = read_hello(&mut conn).await?;
    match hello {
        Hello::ControlChannelHello(service_digest) => {

        }
        Hello::DataChannelHello(_) => {}
    }
    Ok(())
}

async fn do_control_channel_handshake<T:'static + Transport>(
    mut conn: T::Stream,
    addr: SocketAddr,
    services: Arc<RwLock<HashMap<ServiceDigest, ServerServiceConfig>>>,
    control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
    service_digest: ServiceDigest,
) -> Result<()> {
    println!("Now control channel incoming from {}", addr);

    let mut nonce = vec![0u8;HASH_WIDTH_IN_BYTES];


    Ok(())
}