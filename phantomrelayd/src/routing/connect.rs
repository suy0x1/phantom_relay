use anyhow::Result;
use tokio::net::TcpStream;

use crate::{routing::proxy::ProxyProvider, subsystems::rotation::route::RouteContext};

use super::types::socks5::Socks5Proxy;


pub struct Conn {
    pub host: String,
    pub port: u16,
    pub stream: TcpStream
}

pub async fn connect_target(current: RouteContext, host: &str, port: u16) -> Result<Conn> {
    let proxy = Socks5Proxy {
        proxy_addr: format!("{}:{}",current.proxy.ip,current.proxy.port),
    };
    let conn = proxy.connect(host, port).await?;

    Ok(Conn {
        host: format!("{}",current.proxy.ip),
        port: current.proxy.port.clone(),
        stream: conn,
    })
}