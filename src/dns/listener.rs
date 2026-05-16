use anyhow::Result;

use crate::dns::cache::CacheEntry;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::Semaphore;

pub async fn start_dns_listener(cache: Arc<DashMap<Vec<u8>, CacheEntry>>) -> Result<()> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:9002").await?);
    let limit = Arc::new(Semaphore::new(100));
    let mut buf = [0u8; 4096];
    println!("starting DNS server");
    loop {
        let (size, client_addr) = match socket.recv_from(&mut buf).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("recv error {}", e);
                continue;
            }
        };

        let packet = buf[..size].to_vec();
        let socket = socket.clone();
        if let Some(x) = cache.get(&packet[2..].to_vec()) {
            if Instant::now() < x.expires_at {
                let mut res: Vec<u8> = x.response.to_owned();
                res[0] = packet[0];
                res[1] = packet[1];
                if let Err(e) = socket.send_to(&res, client_addr).await {
                    eprintln!("send error: {}", e);
                }
            }
            else {
                drop(x);
                cache.remove(&packet[2..].to_vec());
            }
        } else {
            let cache_map = cache.clone();
            let permit = limit.clone().acquire_owned().await?;

            tokio::spawn(async move {
                let _permit = permit;

                let response = match crate::dns::doh::forward_dns(packet, cache_map).await {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("dns forward error: {:#?}", e);
                        return;
                    }
                };

                if let Err(e) = socket.send_to(&response, client_addr).await {
                    eprintln!("send error: {}", e);
                }
            });
        }
    }
}
