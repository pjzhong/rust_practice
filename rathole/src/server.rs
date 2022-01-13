use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use bincode::Options;
use rand::RngCore;
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time;

use crate::config::{ServerConfig, ServerServiceConfig, ServiceType, TransportType};
use crate::protocol::{read_auth, read_hello, Ack, ControlChannelCmd, Hello, HASH_WIDTH_IN_BYTES};
use crate::transport::{TcpTransport, Transport};
use crate::{protocol, Config};

type ServiceDigest = protocol::Digest;
type Nonce = protocol::Digest;

const CHAN_SIZE: usize = 2048; // The capacity of various chans

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
    // shutdown the control channel by dropping it
    shutdown_tx: broadcast::Sender<bool>,
    data_channel_tx: mpsc::Sender<T::Stream>,
}

impl<T> ControlChannelHandle<T>
where
    T: 'static + Transport,
{
    fn run(conn: T::Stream, service: ServerServiceConfig) -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<bool>(1);
        let (data_ch_tx, data_ch_rx) = mpsc::channel(CHAN_SIZE * 2);
        let (data_ch_req_tx, data_ch_req_rx) = mpsc::unbounded_channel();

        match service.service_type {
            ServiceType::Tcp => tokio::spawn(run_tcp_connection_pool::<T>(
                service.bind_addr.clone(),
                data_ch_rx,
                data_ch_req_tx.clone(),
                shutdown_tx.subscribe(),
            )),
        };

        tokio::spawn(async move {
            if let Err(err) =
                ControlChannelHandle::<T>::do_run(conn, service, shutdown_rx, data_ch_req_rx)
                    .await
                    .with_context(|| "Failed to write data cmds")
            {
                eprintln!("{:?}", err);
            }
        });

        Self {
            shutdown_tx,
            data_channel_tx: data_ch_tx,
        }
    }

    async fn do_run(
        mut conn: T::Stream,
        service: ServerServiceConfig,
        mut shutdown_rx: broadcast::Receiver<bool>,
        mut data_ch_req_rx: mpsc::UnboundedReceiver<bool>,
    ) -> Result<()>
    where
        T: Transport,
    {
        let cmd = bincode::serialize(&ControlChannelCmd::CreateDataChannel).unwrap();

        loop {
            tokio::select! {
                val = data_ch_req_rx.recv() => {
                    match val {
                        Some(_) => {
                            if let Err(e) = conn.write_all(&cmd).await.with_context(||"Failed to write data cmds") {
                                eprintln!("{:?}", e);
                                break;
                            }
                        }
                        None => {
                            break;
                        }
                    }
                },
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        println!("Control channel shutting down");
        Ok(())
    }
}

async fn run_tcp_connection_pool<T: Transport>(
    bind_addr: String,
    mut data_ch_rx: mpsc::Receiver<T::Stream>,
    data_ch_req_tx: mpsc::UnboundedSender<bool>,
    shutdown_rx: broadcast::Receiver<bool>,
) -> Result<()> {
    Ok(())
}

fn tcp_listen_and_send(
    add: String,
    data_ch_req_tx: mpsc::UnboundedSender<bool>,
    mut shutdown_rx: broadcast::Receiver<bool>,
) {
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

impl<'a, T: 'static + Transport> Server<'a, T> {
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
                                if let Err(err) = handle_connection(conn, addr, services, control_channels)
                                .await
                                .with_context(||"Failed to handle a connection to `server.bind_addr`") {
                                    eprintln!("{:?}", err);
                                }
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
            do_control_channel_handshake(conn, addr, services, control_channels, service_digest)
                .await?;
        }
        Hello::DataChannelHello(nonce) => {
            do_data_channel_handshake(conn, control_channels, nonce).await?;
        }
    }
    Ok(())
}

async fn do_control_channel_handshake<T: 'static + Transport>(
    mut conn: T::Stream,
    addr: SocketAddr,
    services: Arc<RwLock<HashMap<ServiceDigest, ServerServiceConfig>>>,
    control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
    service_digest: ServiceDigest,
) -> Result<()> {
    println!("New control channel incoming from {}", addr);

    let mut nonce = vec![0u8; HASH_WIDTH_IN_BYTES];
    rand::thread_rng().fill_bytes(&mut nonce);

    let nonce_send = Hello::ControlChannelHello(nonce.clone().try_into().unwrap());
    conn.write_all(&bincode::serialize(&nonce_send).unwrap())
        .await?;

    let services_guard = services.read().await;
    let services_config = match services_guard.get(&service_digest) {
        Some(v) => v,
        None => {
            conn.write_all(&bincode::serialize(&Ack::ServiceNotExist).unwrap())
                .await?;
            bail!("No such a service {}", hex::encode(&service_digest));
        }
    };

    let protocol::Auth(d) = read_auth(&mut conn).await?;

    let service_name = &services_config.name;

    let mut concat = Vec::from(services_config.token.as_ref().unwrap().as_bytes());
    concat.append(&mut nonce);

    let session_key = protocol::digest(&concat);
    if session_key != d {
        conn.write_all(&bincode::serialize(&Ack::AuthFailed).unwrap())
            .await?;
        println!(
            "Expect {}, but got {}",
            hex::encode(session_key),
            hex::encode(d)
        );

        bail!("Service {} failed the authentication", service_name);
    } else {
        let service_config = services_config.clone();
        drop(services_guard);

        let mut h = control_channels.write().await;

        if h.remove(&service_digest).is_some() {
            eprintln!(
                "Dropping previous control channel for digest {}",
                hex::encode(service_digest)
            );
        }

        conn.write_all(&bincode::serialize(&Ack::Ok).unwrap())
            .await?;

        println!("{} control channel established", service_config.name);

        let handle = ControlChannelHandle::run(conn, service_config);

        let _ = h.insert(service_digest, handle);
    }

    Ok(())
}

async fn do_data_channel_handshake<T: 'static + Transport>(
    conn: T::Stream,
    control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
    nonce: Nonce,
) -> Result<()> {
    let control_channels_guard = control_channels.read().await;
    match control_channels_guard.get(&nonce) {
        Some(handle) => {
            handle.data_channel_tx.send(conn).await?;
        }
        None => {
            eprintln!("Data channels has incorrect nonce");
        }
    }

    Ok(())
}
