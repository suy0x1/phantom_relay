use anyhow::Result;
use std::sync::Arc;

use super::handler::handle_client;
use crate::config::proxy::ProxyConfig;
use crate::monitor::events::Event::{Error, ServiceStartup};
use crate::routing::manager::ConnectionManager;
use chrono::Local;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::monitor::bus::Bus;

pub async fn start_socks5_server(
    config: Arc<ProxyConfig>,
    conn_map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
    cancel: CancellationToken,
) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    bus.emit(ServiceStartup {
        service_name: "SOCKS5 Proxy Server".to_string(),
        port: config.port,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;

    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                break;
            }

            result = listener.accept() => {
                let (stream, _) = result?;

                let conn_map_clone =
                    conn_map.clone();

                let bus = bus.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_client(
                        stream,
                        conn_map_clone.clone(),
                        bus.clone(),
                    )
                    .await
                    {
                        let _ = bus.emit(Error {
                            err: format!("{}", e),
                            timestamp: Local::now()
                                .format("%H:%M:%S")
                                .to_string()
                                .to_string(),
                        });
                    }
                });
            }
        }
    }

    Ok(())
}
