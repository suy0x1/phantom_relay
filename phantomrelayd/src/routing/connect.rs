use anyhow::Result;
use tokio::net::TcpStream;

use crate::{
    monitor::bus::Bus, monitor::error_ext::BusErrorExt, routing::proxy::ProxyProvider,
    subsystems::rotation::route::RouteContext,
};

use super::types::socks5::Socks5Proxy;
use std::sync::Arc;

pub struct Conn {
    pub host: String,
    pub port: u16,
    pub stream: TcpStream,
}

/// Connects to a target host through a proxy selected from the route context.
pub async fn connect_target(
    current: RouteContext,
    host: &str,
    port: u16,
    bus: Arc<Bus>,
) -> Result<Conn> {
    let proxy = Socks5Proxy {
        proxy_addr: format!("{}:{}", current.proxy.ip, current.proxy.port),
    };
    let conn = proxy.connect(host, port).await.emit_to_bus(&bus)?;

    Ok(Conn {
        host: format!("{}", current.proxy.ip),
        port: current.proxy.port.clone(),
        stream: conn,
    })
}
