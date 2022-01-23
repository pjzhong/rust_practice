use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use rand::RngCore;
use tokio::io::{self, copy_bidirectional, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time;
use tracing::{debug, error, info, info_span, instrument, Instrument, Span};

use crate::config::{ServerConfig, ServerServiceConfig, ServiceType, TransportType};
use crate::protocol::{
    read_auth, read_hello, Ack, ControlChannelCmd, DataChannelCmd, Hello, HASH_WIDTH_IN_BYTES,
};
use crate::transport::{TcpTransport, Transport};
use crate::{protocol, Config};

type ServiceDigest = protocol::Digest;

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
    _shutdown_tx: broadcast::Sender<bool>,
    data_channel_tx: mpsc::Sender<T::Stream>,
}

impl<T> ControlChannelHandle<T>
where
    T: 'static + Transport,
{
    /// Create a control channel handle, where the control channel handling task
    /// and the connection pool task are created.

    fn run(
        conn: T::Stream,
        service: ServerServiceConfig,
        service_digest: ServiceDigest,
        control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
    ) -> Self {
        info!("control channel established");

        let (shutdown_tx, shutdown_rx) = broadcast::channel::<bool>(1);
        let (data_ch_tx, data_ch_rx) = mpsc::channel(CHAN_SIZE * 2);
        let (data_ch_req_tx, data_ch_req_rx) = mpsc::unbounded_channel();

        match service.service_type {
            ServiceType::Tcp => tokio::spawn(run_tcp_connection_pool::<T>(
                service.name.clone(),
                service.bind_addr.clone(),
                data_ch_rx,
                data_ch_req_tx,
                shutdown_tx.subscribe(),
            )),
        };

        tokio::spawn(
            async move {
                if let Err(err) =
                    ControlChannelHandle::<T>::do_run(conn, service, shutdown_rx, data_ch_req_rx)
                        .await
                        .with_context(|| "Failed to write data cmds")
                {
                    error!("{:?}", err);
                }

                let mut write_guard = control_channels.write().await;
                write_guard.remove(&service_digest);
                info!("Control channel shutting down");
            }
            .instrument(Span::current()),
        );

        Self {
            _shutdown_tx: shutdown_tx,
            data_channel_tx: data_ch_tx,
        }
    }

    // Run a control channel
    #[instrument(skip_all, fields(service = % service.name))]
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
                Ok(Hello::ControlChannelClose(_)) = read_hello(&mut conn) => {
                    break;
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }

        Ok(())
    }
}

#[instrument(skip_all, fields(service = %name))]
async fn run_tcp_connection_pool<T: 'static + Transport>(
    name: String,
    bind_addr: String,
    mut data_ch_rx: mpsc::Receiver<T::Stream>,
    data_ch_req_tx: mpsc::UnboundedSender<bool>,
    shutdown_rx: broadcast::Receiver<bool>,
) -> Result<()> {
    let mut stream_rx = tcp_listen_and_service_bind(name, bind_addr, data_ch_req_tx, shutdown_rx);
    while let Some(mut steam) = stream_rx.recv().await {
        if let Some(mut ch) = data_ch_rx.recv().await {
            tokio::spawn(
                async move {
                    let cmd = bincode::serialize(&DataChannelCmd::StartForwardTcp).unwrap();
                    if ch.write_all(&cmd).await.is_ok() {
                        info!("start forwarding");
                        let _ = copy_bidirectional(&mut ch, &mut steam).await;
                    }
                }
                .instrument(Span::current()),
            );
        } else {
            break;
        }
    }
    info!("tcp pool close");
    Ok(())
}

///监听对应tcp端口并绑定服务
#[instrument(skip_all, fields(service = %name))]
fn tcp_listen_and_service_bind(
    name: String,
    addr: String,
    data_ch_req_tx: mpsc::UnboundedSender<bool>,
    mut shutdown_rx: broadcast::Receiver<bool>,
) -> mpsc::Receiver<TcpStream> {
    let (tx, rx) = mpsc::channel(CHAN_SIZE);

    tokio::spawn(
        async move {
            let listener = backoff::future::retry(
                ExponentialBackoff {
                    max_elapsed_time: None,
                    max_interval: Duration::from_secs(1),
                    ..Default::default()
                },
                || async { Ok(TcpListener::bind(&addr).await?) },
            )
            .await
            .with_context(|| format!("Failed to listen for the service on addr:{:?}", addr));

            info!("Listening at:{:?}", addr);

            let listener: TcpListener = match listener {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("{:?}", e);
                    return;
                }
            };

            // Retry at least every 1s
            let mut backoff = ExponentialBackoff {
                max_interval: Duration::from_secs(1),
                max_elapsed_time: None,
                ..Default::default()
            };

            loop {
                tokio::select! {
                    val = listener.accept() => {
                        match val {
                            Err(e) => {
                                error!("{}. Sleep for a while", e);
                                if let Some(d) = backoff.next_backoff() {
                                    time::sleep(d).await;
                                } else {
                                      // This branch will never be reached for current backoff policy
                                    error!("Too many retries. Aborting...");
                                    break;
                                }
                            }
                            Ok((incoming, addr)) => {
                                if let Err(e) = data_ch_req_tx.send(true)
                                .with_context(|| "Failed to send data channel create request") {
                                    error!("{:?}", e);
                                    break;
                                }

                                backoff.reset();

                                info!("New visitor from {}", addr);

                                let _ = tx.send(incoming).await;
                            }
                        }
                    },
                    _ = shutdown_rx.recv() => {
                        info!("close");
                        break;
                    }
                }
            }
        }
        .instrument(Span::current()),
    );

    rx
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

        info!("Listening at {}", self.config.bind_addr);

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
                            if err.downcast_ref::<io::Error>().is_some() {
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

                            let services = self.services.clone();
                            let control_channels = self.control_channels.clone();
                            tokio::spawn(async move {
                                info!("Handling");
                                if let Err(err) = handle_connection(conn, addr, services, control_channels)
                                .await
                                .with_context(|| format!("Failed to handle connection")) {
                                    error!("{:?}", err);
                                }
                            }.instrument(info_span!("handle_connection", %addr)));
                        }
                    }
                },
                _ = shutdown_rx.recv() => {
                    info!("Shutting down gracefully...");
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
        _ => {}
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
    info!("New control channel incoming from {}", addr);

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
        debug!(
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

        let handle = ControlChannelHandle::run(
            conn,
            service_config,
            service_digest,
            control_channels.clone(),
        );

        let _ = h.insert(service_digest, handle);
    }

    Ok(())
}

async fn do_data_channel_handshake<T: 'static + Transport>(
    conn: T::Stream,
    control_channels: Arc<RwLock<HashMap<ServiceDigest, ControlChannelHandle<T>>>>,
    nonce: ServiceDigest,
) -> Result<()> {
    let control_channels_guard = control_channels.read().await;
    match control_channels_guard.get(&nonce) {
        Some(handle) => {
            handle.data_channel_tx.send(conn).await?;
            info!("data channel ready")
        }
        None => {
            eprintln!("Data channels has incorrect nonce");
        }
    }

    Ok(())
}
