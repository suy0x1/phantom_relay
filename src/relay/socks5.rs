use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;
use crate::relay::proxy::ProxyProvider;

pub struct Socks5Proxy {
    pub proxy_addr: String,
}

#[async_trait]
impl ProxyProvider for Socks5Proxy {
    async fn connect(
        &self,
        host: &str,
        port: u16
    ) -> Result<TcpStream> {
        let stream = Socks5Stream::connect(&self.proxy_addr.as_str(), (host,port)).await?;
        Ok(stream.into_inner())
    }
}