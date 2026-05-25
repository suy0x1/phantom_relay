use crate::config::tproxy::TProxyConfig;
use crate::monitor::bus::Bus;
use crate::monitor::error_ext::BusErrorExt;
use crate::monitor::events::Event::{DisableCapability, EnableCapability, Error, ServiceStartup, ServiceShutdown};
use crate::routing::manager::ConnectionManager;
use crate::subsystems::network::capablities::NetworkCapability::{
    LocalhostBypass, QUICBlocking, TransparentProxy,
};
use crate::subsystems::rotation::route::RouteContext;
use crate::tproxy::relay::handle_connection;
use anyhow::Result;
use chrono::Local;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tokio::sync::RwLock;

pub async fn start_listener(
    config: Arc<TProxyConfig>,
    conn_map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
    current: Arc<RwLock<RouteContext>>,
    cancel: CancellationToken,
) -> Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .emit_to_bus(&bus)?;

    bus.emit(ServiceStartup {
        service_name: "TCP Proxy Server".to_string(),
        port: config.port,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;

    bus.emit(EnableCapability {
        cap: QUICBlocking,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;
    
    bus.emit(EnableCapability {
        cap: LocalhostBypass,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;

    bus.emit(EnableCapability {
        cap: TransparentProxy,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                bus.emit(DisableCapability {
                    cap: TransparentProxy,
                    timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
                })?;
                bus.emit(DisableCapability {
                    cap: QUICBlocking,
                    timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
                })?;
                bus.emit(DisableCapability {
                    cap: LocalhostBypass,
                    timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
                })?;
                bus.emit(ServiceShutdown {
                    service_name: "TCP Proxy Server".to_string(),
                    port: config.port,
                    timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
                })?;
                break;
            }

            result = listener.accept() => {
                let (stream, _) = result.emit_to_bus(&bus)?;

                let conn_map = conn_map.clone();
                let bus_clone = bus.clone();
                let current = current.read().await.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(
                        current,
                        stream,
                        conn_map,
                        bus_clone.clone(),
                    )
                    .await
                    {
                        let _ = bus_clone.emit(Error {
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
