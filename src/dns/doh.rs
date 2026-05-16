use anyhow::Result;
use reqwest::Client;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::dns::cache::CacheEntry;
use dashmap::DashMap;
use std::sync::Arc;

static DOH_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .proxy(reqwest::Proxy::https("socks5h://127.0.0.1:9050").expect("valid proxy"))
        .timeout(Duration::from_secs(3))
        .build()
        .expect("client build failed")
});

pub async fn forward_dns(
    packet: Vec<u8>,
    cache: Arc<DashMap<Vec<u8>, CacheEntry>>,
) -> Result<Vec<u8>> {
    let key = packet[2..].to_vec();
    let response = DOH_CLIENT
        .post("https://cloudflare-dns.com/dns-query")
        .header("content-type", "application/dns-message")
        .header("accept", "application/dns-message")
        .body(packet)
        .send()
        .await?;
    let res_ret = response.bytes().await?.to_vec();
    let res = res_ret.clone();

    let value = CacheEntry {
        response: res,
        expires_at: Instant::now() + Duration::from_secs(90),
    };

    cache.insert(key, value);

    Ok(res_ret)
}
