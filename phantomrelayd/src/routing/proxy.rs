use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;

#[async_trait]
pub trait ProxyProvider {
    async fn connect(&self, host: &str, port: u16) -> Result<TcpStream>;
}
