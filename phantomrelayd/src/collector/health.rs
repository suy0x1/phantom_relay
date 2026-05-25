use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::collector::manager::HealthyProxy;

use super::manager::PorxyMetadata;
use anyhow::Result;
use dashmap::DashMap;
use reqwest::{Client, Proxy};
use tokio_util::sync::CancellationToken;

async fn check_info(proxy: &str) -> Result<(PorxyMetadata, Client)> {
    let proxy = proxy.replace("socks5://", "socks5h://");
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
    cancel: CancellationToken,
) -> Result<()> {
    for i in proxies {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let start = Instant::now();

        let res = tokio::select! {
            _ = cancel.cancelled() => {
                return Ok(());
            }

            res = check_info(&i) => res
        };

        match res {
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
    }

    Ok(())
}
