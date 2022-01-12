use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, oneshot};
use tokio::time;

use crate::{Config, protocol};
use crate::config::{ClientConfig, ClientServiceConfig, TransportType};
use crate::protocol::{Hello, read_hello};
use crate::transport::{TcpTransport, Transport};

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

struct ControlChannel<T: Transport> {
    digest: ServiceDigest,
    service: ClientServiceConfig,
    shutdown_rx: oneshot::Receiver<u8>,
    remote_addr: String,
    transport: Arc<T>,
}

impl<T: 'static + Transport> ControlChannel<T> {
    async fn run(&mut self) -> Result<()> {
        let mut conn_control = self
            .transport
            .connect(&self.remote_addr)
            .await
            .with_context(|| format!("Failed to connect to the server: {}", &self.remote_addr))?;

        //send hello
        let hello_send = Hello::ControlChannelHello(self.digest[..].try_into().unwrap());
        conn_control
            .write_all(&bincode::serialize(&hello_send).unwrap())
            .await?;

        //reading hello
        let nonce = match protocol::read_hello(&mut conn_control)
            .await
            .with_context(|| "Failed to read hello from the sever")? {
            Hello::ControlChannelHello(d) => d,
            _ => {
                bail!("Unexpected type of hello");
            }
        };

        //TODO read hello

        //TODO send auth

        //TODO Read ack

        //TODO channel ready

        loop {
            tokio::select! {
                _ = &mut self.shutdown_rx => {
                    println!("Control channel shutting down..");
                    break;
                }
            }
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
    fn run<T: 'static + Transport>(
        service: ClientServiceConfig,
        remote_addr: String,
        transport: Arc<T>,
    ) -> Self {
        let digest = protocol::digest(service.name.as_bytes());

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

                let duration = Duration::from_secs(1);
                eprintln!("{:?}\n\nRetry in {:?}...", err, duration);
                time::sleep(duration).await;
            }
        });

        Self { shutdown_tx }
    }

    fn shutdown(self) {
        let _ = self.shutdown_tx.send(0u8);
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
                            eprintln!("Unable to listen for shutdown signal: {}", err);
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
