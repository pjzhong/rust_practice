use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use backoff::ExponentialBackoff;
use tokio::io::{copy_bidirectional, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, oneshot};
use tokio::time;
use tracing::{debug, error, info, instrument, Instrument, Span};

use crate::config::{ClientConfig, ClientServiceConfig, TransportType};
use crate::protocol::{
    read_ack, read_control_cmd, read_data_cmd, Ack, Auth, ControlChannelCmd, DataChannelCmd, Hello,
    HASH_WIDTH_IN_BYTES,
};
use crate::transport::{TcpTransport, Transport};
use crate::{protocol, Config};

type ServiceDigest = protocol::Digest;

pub async fn run_client(config: &Config, shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
    let config = match &config.client {
        Some(v) => v,
        None => {
            return Err(anyhow!("Try to run as a client, but the configuration is missing. Please add the `[client]` block"))
        }
    };

    match config.transport {
        TransportType::Tcp => {
            let mut client = Client::<TcpTransport>::from(config).await?;
            client.run(shutdown_rx).await
        }
    }
}

struct Client<'a, T: Transport> {
    config: &'a ClientConfig,
    service_handles: HashMap<String, ControlChannelHandle>,
    transport: Arc<T>,
}

impl<'a, T: 'static + Transport> Client<'a, T> {
    async fn from(config: &'a ClientConfig) -> Result<Client<'a, T>> {
        Ok(Client {
            config,
            service_handles: HashMap::new(),
            transport: Arc::new(
                T::new(&config.transport)
                    .await
                    .with_context(|| "Failed to create the transport")?,
            ),
        })
    }

    async fn run(&mut self, mut shutdown_rx: broadcast::Receiver<bool>) -> Result<()> {
        for (name, config) in &self.config.services {
            let handle = ControlChannelHandle::run(
                (*config).clone(),
                self.config.remote_addr.clone(),
                self.transport.clone(),
            );
            self.service_handles.insert(name.clone(), handle);
        }

        loop {
            tokio::select! {
                val = shutdown_rx.recv() => {
                    match val {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Unable to listen for shutdown signal: {}", err);
                        }
                    }
                    break;
                },
            }
        }

        for (_, handle) in self.service_handles.drain() {
            handle.shutdown();
        }

        Ok(())
    }
}

// Handle of a control channel
// Dropping it will also drop the actual control channel
struct ControlChannelHandle {
    shutdown_tx: oneshot::Sender<u8>,
}

impl ControlChannelHandle {
    #[instrument(skip_all, fields(service = %service.name))]
    fn run<T: 'static + Transport>(
        service: ClientServiceConfig,
        remote_addr: String,
        transport: Arc<T>,
    ) -> Self {
        let digest = protocol::digest(service.name.as_bytes());

        info!("Starting {}", hex::encode(digest));
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let mut control_channel = ControlChannel {
            digest,
            service,
            shutdown_rx,
            remote_addr,
            transport,
        };

        tokio::spawn(async move {
            while let Err(err) = control_channel
                .run()
                .await
                .with_context(|| "Failed to run the control channel")
            {
                if control_channel.shutdown_rx.try_recv()
                    != Err(oneshot::error::TryRecvError::Empty)
                {
                    break;
                }

                let duration = Duration::from_secs(3);
                error!("{:?}\n\nRetry in {:?}...", err, duration);
                time::sleep(duration).await;
            }
        });

        Self { shutdown_tx }
    }

    fn shutdown(self) {
        let _ = self.shutdown_tx.send(0u8);
    }
}

struct ControlChannel<T: Transport> {
    digest: ServiceDigest,
    service: ClientServiceConfig,
    shutdown_rx: oneshot::Receiver<u8>,
    remote_addr: String,
    transport: Arc<T>,
}

impl<T: 'static + Transport> ControlChannel<T> {
    #[instrument(skip(self), fields(service = %self.service.name))]
    async fn run(&mut self) -> Result<()> {
        let mut conn_control = self
            .transport
            .connect(&self.remote_addr)
            .await
            .with_context(|| format!("Failed to connect to the server: {}", &self.remote_addr))?;

        //send hello
        debug!("Sending hello");
        let hello_send = Hello::ControlChannelHello(self.digest[..].try_into().unwrap());
        conn_control
            .write_all(&bincode::serialize(&hello_send).unwrap())
            .await?;

        //reading hello
        debug!("Reading hello");
        let nonce = match protocol::read_hello(&mut conn_control)
            .await
            .with_context(|| "Failed to read hello from the sever")?
        {
            Hello::ControlChannelHello(d) => d,
            _ => {
                bail!("Unexpected type of hello");
            }
        };

        // Read Ack
        debug!("Sending auth");
        let mut concat = Vec::from(self.service.token.as_ref().unwrap().as_bytes());
        concat.extend_from_slice(&nonce);

        let session_key = protocol::digest(&concat);
        let auth = Auth(session_key);
        conn_control
            .write_all(&bincode::serialize(&auth).unwrap())
            .await?;

        //Read ack
        debug!("Reading ack");
        match read_ack(&mut conn_control).await? {
            Ack::Ok => {
                info!("Authentication success")
            }
            v => {
                return Err(anyhow!("{:?}", v))
                    .with_context(|| format!("Authentication failed: {}", self.service.name));
            }
        }

        info!("Control channel established");

        let remote_addr = self.remote_addr.clone();
        let local_addr = self.service.local_addr.clone();
        let session_key = self.digest;
        let data_ch_args = Arc::new(RunDataChannelArgs {
            session_key,
            remote_addr,
            local_addr,
            connector: self.transport.clone(),
        });

        loop {
            tokio::select! {
                val = read_control_cmd(&mut conn_control) => {
                    let val = val?;
                    match val {
                        ControlChannelCmd::CreateDataChannel => {
                            let args = data_ch_args.clone();
                            tokio::spawn(async move {
                                if let Err(e) = run_data_channel(args).await.with_context(|| "Failed to run the data channel") {
                                    error!("{:?}", e);
                                }
                            }.instrument(Span::current()));
                        }
                    }
                },
                _ = &mut self.shutdown_rx => {
                    let close_send = Hello::ControlChannelClose(self.digest[..].try_into().unwrap());
                    conn_control.write_all(&bincode::serialize(&close_send).unwrap()).await?;
                    println!("Control channel shutting down..");
                    break;
                }
            }
        }

        Ok(())
    }
}

struct RunDataChannelArgs<T: Transport> {
    session_key: ServiceDigest,
    remote_addr: String,
    local_addr: String,
    connector: Arc<T>,
}

async fn run_data_channel<T: Transport>(args: Arc<RunDataChannelArgs<T>>) -> Result<()> {
    let mut conn = do_data_channel_handshake(args.clone()).await?;

    info!("new data channel created waiting");
    match read_data_cmd(&mut conn).await? {
        DataChannelCmd::StartForwardTcp => {
            run_data_channel_for_tcp::<T>(conn, &args.local_addr).await?;
        }
        DataChannelCmd::StartForwardUdp => {
            panic!("Forward udp is not support")
        }
    }
    Ok(())
}

async fn do_data_channel_handshake<T: Transport>(
    args: Arc<RunDataChannelArgs<T>>,
) -> Result<T::Stream> {
    let backoff = ExponentialBackoff {
        max_interval: Duration::from_millis(100),
        max_elapsed_time: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    let mut conn: T::Stream = backoff::future::retry(backoff, || async {
        Ok(args
            .connector
            .connect(&args.remote_addr)
            .await
            .with_context(|| "data channel Failed to connect to remote_addr")?)
    })
    .await?;

    let v: &[u8; HASH_WIDTH_IN_BYTES] = args.session_key[..].try_into().unwrap();
    let hello = Hello::DataChannelHello(v.to_owned());
    conn.write_all(&bincode::serialize(&hello).unwrap()).await?;

    Ok(conn)
}

async fn run_data_channel_for_tcp<T: Transport>(
    mut conn: T::Stream,
    local_addr: &str,
) -> Result<()> {
    let mut local = TcpStream::connect(local_addr)
        .await
        .with_context(|| "Failed to connect to local_addr")?;
    let _ = copy_bidirectional(&mut conn, &mut local).await;

    info!("New data channel starts forwarding to {:?}", local_addr);

    Ok(())
}
