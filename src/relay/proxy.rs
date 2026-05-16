use anyhow::Result;
use tokio::net::TcpStream;
use async_trait::async_trait;

#[async_trait]
pub trait ProxyProvider {
    async fn connect(
        &self,
        host: &str,
        port: u16,
    ) -> Result<TcpStream>;
}