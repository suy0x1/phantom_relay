use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::monitor::{bus::Bus, events::Event};

pub async fn start_logger(bus: Arc<Bus>) -> Result<()> {
    let mut rx = bus.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => match event {
                Event::ServiceStartup {
                    service_name,
                    port,
                    timestamp,
                } => {
                    println!(
                        "[{}] [service] {} started on port {}",
                        timestamp, service_name, port
                    );
                }

                Event::NetworkChange { change, timestamp } => {
                    println!("[{}] [network] {}", timestamp, change);
                }

                Event::ConnectionOpened {
                    host,
                    port,
                    proxy,
                    proxy_port,
                    timestamp,
                } => {
                    println!(
                        "[{}] [conn] opened {}:{} via {}:{}",
                        timestamp, host, port, proxy,proxy_port
                    );
                }

                Event::ConnectionClosed {
                    host,
                    port,
                    proxy,
                    proxy_port,
                    timestamp,
                } => {
                    println!(
                        "[{}] [conn] closed {}:{} via {}:{}",
                        timestamp, host, port, proxy, proxy_port
                    );
                }

                Event::DNSRequest {
                    domain,
                    resolver,
                    timestamp,
                } => {
                    println!(
                        "[{}] [dns] request {} via {}",
                        timestamp, domain, resolver
                    );
                }

                Event::DNSCacheHit { domain, timestamp } => {
                    println!("[{}] [dns] cache hit {}", timestamp, domain);
                }

                Event::DNSCacheMiss { domain, timestamp } => {
                    println!("[{}] [dns] cache miss {}", timestamp, domain);
                }

                Event::ProxyConnected {
                    host,
                    port,
                    timestamp,
                } => {
                    println!("[{}] [proxy] connected {}:{}", timestamp, host, port);
                }

                Event::ProxyFailed {
                    host,
                    port,
                    timestamp,
                } => {
                    println!("[{}] [proxy] failed {}:{}", timestamp, host, port);
                }

                Event::RoutingDecision => {
                    println!("[routing] decision made");
                }

                Event::Error {
                    err,
                    timestamp,
                } => {
                    println!("[{}] [error] {}",timestamp,err);
                }
            },

            Err(broadcast::error::RecvError::Lagged(n)) => {
                eprintln!("logger lagged {} events", n)
            }

            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }
}
