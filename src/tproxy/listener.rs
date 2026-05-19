use crate::config::tproxy::TProxyConfig;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{Error, ServiceStartup};
use crate::tproxy::relay::handle_connection;
use crate::routing::manager::ConnectionManager;
use anyhow::Result;
use std::sync::Arc;
use chrono::Local;
use tokio::net::TcpListener;

pub async fn start_listener(config: Arc<TProxyConfig>, conn_map: Arc<ConnectionManager>, bus: Arc<Bus>) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}",config.host, config.port)).await?;
    bus.emit(ServiceStartup {
        service_name: "TCP Proxy Server".to_string(),
        port: config.port,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;

    loop {
        let (stream, _) = listener.accept().await?;

        let conn_map = conn_map.clone();
        let bus_clone = bus.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, conn_map, bus_clone.clone()).await {
                let _ = bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()});
            }
        });
    }
}
