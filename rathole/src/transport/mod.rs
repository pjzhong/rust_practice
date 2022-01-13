mod tcp;
pub use tcp::TcpTransport;

use std::fmt::Debug;
use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::ToSocketAddrs;

use crate::config::TransportType;

#[async_trait]
pub trait Transport: Debug + Send + Sync {
    type Acceptor: Send + Sync;
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + Sync + Debug;

    async fn new(transport_type: &TransportType) -> Result<Self>
    where
        Self: Sized;
    async fn bind<T: ToSocketAddrs + Send + Sync>(&self, addr: T) -> Result<Self::Acceptor>;
    async fn accept(&self, a: &Self::Acceptor) -> Result<(Self::Stream, SocketAddr)>;
    async fn connect(&self, addr: &str) -> Result<Self::Stream>;
}
