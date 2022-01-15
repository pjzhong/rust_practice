use std::time::Duration;

use anyhow::{Context, Result};
use socket2::{SockRef, TcpKeepalive};
use tokio::net::TcpStream;

pub fn try_set_tcp_keep_alive(conn: &TcpStream) -> Result<()> {
    let s = SockRef::from(conn);
    let keep_alive = TcpKeepalive::new().with_time(Duration::from_secs(60));
    s.set_tcp_keepalive(&keep_alive)
        .with_context(|| "Failed to set keep alive")
}

pub fn set_tcp_keep_alive(conn: &TcpStream) {
    if try_set_tcp_keep_alive(conn).is_err() {
        //TODO ! logging
    }
}

#[allow(dead_code)]
pub fn feature_not_compile(feature: &str) -> ! {
    panic!(
        "The feature '{}' is not compiled in this binary. Please re-compile rathole",
        feature
    )
}
