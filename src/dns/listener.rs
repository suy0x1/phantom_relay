use anyhow::Result;

use crate::config::dns::DNSConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::parse::extract_cache_key;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{DNSCacheHit, DNSCacheMiss, DNSRequest, Error, ServiceStartup};

use chrono::Local;
use dashmap::DashMap;

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::{Notify, Semaphore};

pub async fn start_dns_listener(
    config: Arc<DNSConfig>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    bus: Arc<Bus>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
) -> Result<()> {
    let socket = Arc::new(UdpSocket::bind(format!("{}:{}",config.host,config.port)).await?);

    let limit = Arc::new(Semaphore::new(config.max_parallel_dns_lookups));

    let mut buf = [0u8; 4096];

    bus.emit(ServiceStartup {
        service_name: "DNS server".to_string(),
        port: config.port,
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    })?;

    loop {
        let bus_clone = bus.clone();

        let (size, client_addr) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,

            Err(e) => {
                bus_clone.emit(Error {
                    err: format!("{}", e),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                })?;

                continue;
            }
        };

        let packet = buf[..size].to_vec();

        let key = match extract_cache_key(&packet) {
            Some(v) => v,

            None => {
                continue;
            }
        };

        bus_clone.emit(DNSRequest {
            domain: key.domain.clone(),
            resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
        })?;

        let socket_clone = socket.clone();

        /*
         * FAST PATH
         */

        if let Some(x) = cache.get(&key) {
            if Instant::now() < x.expires_at {
                bus_clone.emit(DNSCacheHit {
                    domain: key.domain.clone(),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                })?;
                x.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let mut res = x.response.clone();

                drop(x);

                res[0] = packet[0];
                res[1] = packet[1];

                if let Err(e) = socket_clone.send_to(&res, client_addr).await {
                    bus_clone.emit(Error {
                        err: format!("{}", e),
                        timestamp: Local::now().format("%H:%M:%S").to_string(),
                    })?;
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
                    bus_clone.emit(DNSCacheHit {
                        domain: key.domain.clone(),
                        timestamp: Local::now().format("%H:%M:%S").to_string(),
                    })?;
                    x.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    let mut res = x.response.clone();

                    drop(x);

                    res[0] = packet[0];
                    res[1] = packet[1];

                    if let Err(e) = socket_clone.send_to(&res, client_addr).await {
                        bus_clone.emit(Error {
                            err: format!("{}", e),
                            timestamp: Local::now().format("%H:%M:%S").to_string(),
                        })?;
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

        bus_clone.emit(DNSCacheMiss {
            domain: key.domain.clone(),
            timestamp: Local::now().format("%H:%M:%S").to_string(),
        })?;

        let notify = Arc::new(Notify::new());

        inflight.insert(key.clone(), notify.clone());

        let cache_clone = cache.clone();

        let inflight_clone = inflight.clone();

        let permit = limit.clone().acquire_owned().await?;

        tokio::spawn(async move {
            let _permit = permit;

            let response = match crate::dns::doh::forward_dns(
                packet,
                cache_clone,
                inflight_clone.clone(),
                notify.clone(),
            )
            .await
            {
                Ok(v) => v,

                Err(e) => {
                    let _ = bus_clone.emit(Error {
                        err: format!("{}", e),
                        timestamp: Local::now().format("%H:%M:%S").to_string(),
                    });

                    inflight_clone.remove(&key);

                    notify.notify_waiters();

                    return;
                }
            };

            if let Err(e) = socket_clone.send_to(&response, client_addr).await {
                let _ = bus_clone.emit(Error {
                    err: format!("{}", e),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                });
            }
        });
    }
}
