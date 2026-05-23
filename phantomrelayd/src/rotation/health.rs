use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use crate::rotation::manager::HealthyProxy;

use super::manager::PorxyMetadata;
use anyhow::Result;
use dashmap::DashMap;
use reqwest::{Client, Proxy};

async fn check_info(proxy: &str) -> Result<(PorxyMetadata, Client)> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .proxy(Proxy::all(proxy)?)
        .build()?;
    let res = client
        .get("https://freeipapi.com/api/json")
        .header("accept", "application/json")
        .send()
        .await?;
    let info: PorxyMetadata = res.json().await?;
    Ok((info, client))
}

pub async fn get_healthy_proxies(
    map: Arc<DashMap<HealthyProxy, Client>>,
    latency: u128,
    proxies: Vec<String>,
    progress: Arc<AtomicU32>,
) -> Result<()> {
    for i in proxies {
        let start = Instant::now();
        match check_info(&i).await {
            Ok(x) => {
                let (ip, port) = i
                    .strip_prefix("socks5://")
                    .unwrap()
                    .split_once(':')
                    .map(|(i, p)| (i, p.parse::<u16>().unwrap()))
                    .unwrap();
                let ms = start.elapsed().as_millis();
                if ms > latency {
                    continue;
                }
                map.insert(
                    HealthyProxy {
                        ip: ip.to_string(),
                        port: port,
                        metadata: x.0,
                        latency: ms,
                    },
                    x.1,
                );
            }
            Err(_) => {}
        }
        progress.fetch_add(1, Ordering::Relaxed);
    }
    Ok(())
}
