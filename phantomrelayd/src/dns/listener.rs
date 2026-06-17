use anyhow::Result;

use crate::config::dns::DNSConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::parse::extract_cache_key;
use crate::monitor::bus::Bus;
use crate::monitor::error_ext::BusErrorExt;
use crate::monitor::events::{CriticalEvent, DiagnosticEvent, LifecycleEvent, TelemetryEvent};
use crate::subsystems::network::capablities::NetworkCapability::DNSIntercept;
use crate::subsystems::rotation::route::RouteContext;

use dashmap::DashMap;

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::sync::{Mutex, Notify, Semaphore};
use tokio_util::sync::CancellationToken;

/// Starts DNS listener on configured host/port, intercepts requests, caches responses, and enforces query limits.
pub async fn start_dns_listener(
    config: Arc<Mutex<DNSConfig>>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    bus: Arc<Bus>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    current: Arc<RwLock<RouteContext>>,
    cancel: CancellationToken,
) -> Result<()> {
    let (host, port, max_par) = {
        let cfg = config.lock().await;
        (cfg.host.clone(), cfg.port, cfg.max_parallel_dns_lookups)
    };
    let socket = Arc::new(
        UdpSocket::bind(format!("{}:{}", host, port))
            .await
            .emit_to_bus(&bus)?,
    );

    let limit = Arc::new(Semaphore::new(max_par));

    let mut buf = [0u8; 4096];

    _ = bus.emit_lifecycle(LifecycleEvent::ServiceStartup {
        service_name: "DNS server".to_string(),
        port: port,
    });

    bus.emit_critical(CriticalEvent::EnableCapability { cap: DNSIntercept })?;
    loop {
        let bus_clone = bus.clone();

        let (size, client_addr) = tokio::select! {

            _ = cancel.cancelled() => {
                bus.emit_critical(CriticalEvent::DisableCapability {
                    cap: DNSIntercept,

                })?;
                _ = bus.emit_lifecycle(LifecycleEvent::ServiceShutdown {
                    service_name: "DNS server".to_string(),
                    port: port,

                });
                break;
            }

            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok(v) => v,

                    Err(e) => {
                        let _ = Err::<(), _>(e).emit_to_bus(&bus_clone);
                        continue;
                    }
                }
            }
        };

        let packet = buf[..size].to_vec();

        let key = match extract_cache_key(&packet) {
            Some(v) => v,

            None => {
                continue;
            }
        };

        bus_clone
            .emit_telemetry(TelemetryEvent::DNSRequest {
                domain: key.domain.clone(),
                resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            })
            .await;

        let socket_clone = socket.clone();

        /*
         * FAST PATH
         */

        if let Some(x) = cache.get(&key) {
            if Instant::now() < x.expires_at {
                bus_clone
                    .emit_telemetry(TelemetryEvent::DNSCacheHit {
                        domain: key.domain.clone(),
                    })
                    .await;

                x.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let mut res = x.response.clone();

                drop(x);

                res[0] = packet[0];
                res[1] = packet[1];

                if let Err(_e) = socket_clone
                    .send_to(&res, client_addr)
                    .await
                    .emit_to_bus(&bus_clone)
                {
                    // Error already emitted via emit_to_bus
                }

                continue;
            }

            drop(x);

            cache.remove(&key);
        }

        /*
         * INFLIGHT WAIT
         */

        if let Some(waiter) = inflight.get(&key) {
            let notify = waiter.clone();

            drop(waiter);

            notify.notified().await;

            if let Some(x) = cache.get(&key) {
                if Instant::now() < x.expires_at {
                    bus_clone
                        .emit_telemetry(TelemetryEvent::DNSCacheHit {
                            domain: key.domain.clone(),
                        })
                        .await;

                    x.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let mut res = x.response.clone();

                    drop(x);

                    res[0] = packet[0];
                    res[1] = packet[1];

                    if let Err(_e) = socket_clone
                        .send_to(&res, client_addr)
                        .await
                        .emit_to_bus(&bus_clone)
                    {
                        // Error already emitted via emit_to_bus
                    }

                    continue;
                }

                drop(x);

                cache.remove(&key);
            }
        }

        /*
         * REAL MISS
         */

        bus_clone
            .emit_telemetry(TelemetryEvent::DNSCacheMiss {
                domain: key.domain.clone(),
            })
            .await;

        let notify = Arc::new(Notify::new());

        inflight.insert(key.clone(), notify.clone());

        let cache_clone = cache.clone();

        let inflight_clone = inflight.clone();

        let permit = limit
            .clone()
            .acquire_owned()
            .await
            .emit_to_bus(&bus_clone)?;

        let client = current.read().await.clone().client;
        tokio::spawn(async move {
            let _permit = permit;
            let response = match crate::dns::doh::forward_dns(
                client,
                packet,
                cache_clone,
                inflight_clone.clone(),
                notify.clone(),
                bus_clone.clone(),
            )
            .await
            {
                Ok(v) => v,

                Err(e) => {
                    _ = bus_clone.emit_diagnostic(DiagnosticEvent::Error {
                        err: format!("{}", e),
                    });

                    _ = bus_clone.emit_critical(CriticalEvent::BadProxy);

                    inflight_clone.remove(&key);

                    notify.notify_waiters();

                    return;
                }
            };

            if let Err(_e) = socket_clone
                .send_to(&response, client_addr)
                .await
                .emit_to_bus(&bus_clone)
            {
                // Error already emitted via emit_to_bus
            }
        });
    }

    Ok(())
}
