use crate::config::tproxy::TProxyConfig;
use crate::monitor::bus::Bus;
use crate::monitor::error_ext::BusErrorExt;
use crate::monitor::events::{CriticalEvent, LifecycleEvent, DiagnosticEvent};
use crate::routing::manager::ConnectionManager;
use crate::subsystems::network::capablities::NetworkCapability::{
    LocalhostBypass, QUICBlocking, TransparentProxy,
};
use crate::subsystems::rotation::route::RouteContext;
use crate::tproxy::relay::handle_connection;
use anyhow::Result;
use std::sync::Arc;
use std::time::SystemTime;
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

    bus.emit_lifecycle(LifecycleEvent::ServiceStartup {
        service_name: "TCP Proxy Server".to_string(),
        port: config.port,
        timestamp: SystemTime::now(),
    }).await;

    bus.emit_critical(CriticalEvent::EnableCapability {
        cap: QUICBlocking,
        timestamp: SystemTime::now(),
    })?;

    bus.emit_critical(CriticalEvent::EnableCapability {
        cap: LocalhostBypass,
        timestamp: SystemTime::now(),
    })?;

    bus.emit_critical(CriticalEvent::EnableCapability {
        cap: TransparentProxy,
        timestamp: SystemTime::now(),
    })?;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                bus.emit_critical(CriticalEvent::DisableCapability {
                    cap: TransparentProxy,
                    timestamp: SystemTime::now(),
                })?;
                bus.emit_critical(CriticalEvent::DisableCapability {
                    cap: QUICBlocking,
                    timestamp: SystemTime::now(),
                })?;
                bus.emit_critical(CriticalEvent::DisableCapability {
                    cap: LocalhostBypass,
                    timestamp: SystemTime::now(),
                })?;
                bus.emit_lifecycle(LifecycleEvent::ServiceShutdown {
                    service_name: "TCP Proxy Server".to_string(),
                    port: config.port,
                    timestamp: SystemTime::now(),
                }).await;
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
                        bus_clone.emit_diagnostic(DiagnosticEvent::Error {
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
