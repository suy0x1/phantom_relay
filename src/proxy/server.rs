use anyhow::Result;
use std::sync::Arc;

use tokio::net::TcpListener;
use super::handler::handle_client;
use crate::monitor::events::Event::{ServiceStartup, Error};
use crate::config::proxy::ProxyConfig;
use crate::routing::manager::ConnectionManager;
use chrono::Local;

use crate::monitor::bus::Bus;

pub async fn start_socks5_server(config: Arc<ProxyConfig>, conn_map: Arc<ConnectionManager>, bus: Arc<Bus>) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}",config.host,config.port)).await?;

    bus.emit(ServiceStartup { service_name: "SOCKS5 Proxy Server".to_string(), port: config.port, timestamp: Local::now().format("%H:%M:%S").to_string().to_string()})?;

    loop {
        let conn_map_clone = conn_map.clone();
        let (stream, _) = listener.accept().await?;

        let bus = bus.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, conn_map_clone.clone(),  bus.clone()).await {
                let _ = bus.emit(Error { err: format!("{}",e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string() });
            }
        });
    }
}