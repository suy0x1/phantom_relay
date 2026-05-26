use anyhow::Result;
use std::sync::Arc;
use std::time::SystemTime;

use super::handler::handle_client;
use crate::config::proxy::ProxyConfig;
use crate::monitor::error_ext::BusErrorExt;
use crate::monitor::events::{LifecycleEvent, DiagnosticEvent};
use crate::routing::manager::ConnectionManager;
use crate::subsystems::rotation::route::RouteContext;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tokio::sync::RwLock;

use crate::monitor::bus::Bus;

pub async fn start_socks5_server(
    config: Arc<ProxyConfig>,
    conn_map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
    current: Arc<RwLock<RouteContext>>,
    cancel: CancellationToken,
) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    bus.emit_lifecycle(LifecycleEvent::ServiceStartup {
        service_name: "SOCKS5 Proxy Server".to_string(),
        port: config.port,
        timestamp: SystemTime::now(),
    }).await;

    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                bus.emit_lifecycle(LifecycleEvent::ServiceShutdown {
                    service_name: "SOCKS5 Proxy Server".to_string(),
                    port: config.port,
                    timestamp: SystemTime::now(),
                }).await;
                break;
            }

            result = listener.accept() => {
                let (stream, _) = result.emit_to_bus(&bus)?;

                let conn_map_clone =
                    conn_map.clone();

                let bus = bus.clone();
                let route = current.read().await.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(
                        route,
                        stream,
                        conn_map_clone.clone(),
                        bus.clone(),
                    )
                    .await
                    {
                        bus.emit_diagnostic(DiagnosticEvent::Error {
                            err: format!("{}", e),
                            timestamp: SystemTime::now(),
                        });
                    }
                });
            }
        }
    }

    Ok(())
}
