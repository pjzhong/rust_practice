use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::config::TransportType;
use crate::helper::set_tcp_keep_alive;
use crate::transport::Transport;

#[derive(Debug)]
pub struct TcpTransport {}

#[async_trait]
impl Transport for TcpTransport {
    type Acceptor = TcpListener;
    type Stream = TcpStream;

    async fn new(_: &TransportType) -> Result<Self> {
        Ok(TcpTransport {})
    }

    async fn bind<T: ToSocketAddrs + Send + Sync>(&self, addr: T) -> Result<Self::Acceptor> {
        Ok(TcpListener::bind(addr).await?)
    }

    async fn accept(&self, a: &Self::Acceptor) -> Result<(Self::Stream, SocketAddr)> {
        let (s, addr) = a.accept().await?;
        set_tcp_keep_alive(&s);
        Ok((s, addr))
    }

    async fn connect(&self, addr: &str) -> Result<Self::Stream> {
        let s = TcpStream::connect(addr).await?;
        set_tcp_keep_alive(&s);
        Ok(s)
    }
}
