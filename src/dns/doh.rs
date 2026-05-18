use anyhow::Result;
use reqwest::Client;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::parse::{extract_cache_key, extract_ips, extract_min_ttl};
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
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    notify: Arc<Notify>,
) -> Result<Vec<u8>> {
    let key =
        extract_cache_key(&packet).ok_or_else(|| anyhow::anyhow!("failed to extract cache key"))?;
    let ttl = extract_min_ttl(&packet[0..]).unwrap_or(90);
    let ips = extract_ips(&packet[0..]);
    let rc = packet[3] & 0x0F;

    let response = DOH_CLIENT
        .post("https://cloudflare-dns.com/dns-query")
        .header("content-type", "application/dns-message")
        .header("accept", "application/dns-message")
        .body(packet)
        .send()
        .await?;

    let response_bytes = response.bytes().await?.to_vec();

    let value = CacheEntry {
        response: response_bytes.clone(),
        expires_at: Instant::now() + Duration::from_secs(ttl as u64),
        resolved_ips: ips,
        hits: 0.into(),
        rcode: rc,
    };

    cache.insert(key.clone(), value);
    notify.notify_waiters();
    inflight.remove(&key);

    Ok(response_bytes)
}
