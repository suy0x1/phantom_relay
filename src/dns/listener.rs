use anyhow::Result;

use crate::dns::cache::CacheEntry;
use crate::dns::parse::extract_domain;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{DNSCacheHit, DNSCacheMiss, DNSRequest, Error, ServiceStartup};
use chrono::Local;
use dashmap::DashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::Semaphore;

pub async fn start_dns_listener(
    cache: Arc<DashMap<Vec<u8>, CacheEntry>>,
    bus: Arc<Bus>,
) -> Result<()> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:9002").await?);
    let limit = Arc::new(Semaphore::new(100));
    let mut buf = [0u8; 4096];
    bus.emit(ServiceStartup {
        service_name: "DNS server".to_string(),
        port: 9002,
        timestamp: Local::now().format("%H:%M:%S").to_string().to_string(),
    })?;
    loop {
        let bus_clone = bus.clone();
        let (size, client_addr) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()})?;
                continue;
            }
        };

        let packet = buf[..size].to_vec();
        if let Some(x) = extract_domain(&packet[..size]) {
            bus_clone.emit(DNSRequest {
                domain: x,
                resolver: IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
                timestamp: Local::now().format("%H:%M:%S").to_string(),
            })?;
        };
        let socket = socket.clone();
        if let Some(x) = cache.get(&packet[2..].to_vec()) {
            if Instant::now() < x.expires_at {
                if let Some(y) = extract_domain(&packet[..size]) {
                    bus_clone.emit(DNSCacheHit {
                        domain: y,
                        timestamp: Local::now().format("%H:%M:%S").to_string(),
                    })?;
                };
                let mut res: Vec<u8> = x.response.to_owned();
                res[0] = packet[0];
                res[1] = packet[1];
                if let Err(e) = socket.send_to(&res, client_addr).await {
                    bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()})?;
                }
            } else {
                drop(x);
                cache.remove(&packet[2..].to_vec());
            }
        } else {
            let cache_map = cache.clone();
            let permit = limit.clone().acquire_owned().await?;
            if let Some(y) = extract_domain(&packet[..size]) {
                bus_clone.emit(DNSCacheMiss {
                    domain: y,
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                })?;
            };

            tokio::spawn(async move {
                let _permit = permit;

                let response = match crate::dns::doh::forward_dns(packet, cache_map).await {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()});
                        return;
                    }
                };

                if let Err(e) = socket.send_to(&response, client_addr).await {
                    let _ = bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()});
                }
            });
        }
    }
}
