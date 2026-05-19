use anyhow::Result;
use tokio::net::TcpStream;

use crate::routing::proxy::ProxyProvider;

use super::types::socks5::Socks5Proxy;


pub struct Conn {
    pub host: String,
    pub port: u16,
    pub stream: TcpStream
}

pub async fn connect_target(host: &str, port: u16) -> Result<Conn> {
    let proxy = Socks5Proxy {
        proxy_addr: "127.0.0.1:9050".to_string(),
    };

    let conn = proxy.connect(host, port).await?;

    Ok(Conn {
        host: "127.0.0.1".to_string(),
        port: 9050,
        stream: conn,
    })
}

pub fn get_dns_proxy() -> String {
    "socks5h://127.0.0.1:9050".to_string()
}