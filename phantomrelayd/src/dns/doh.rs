use anyhow::Result;
use reqwest::Client;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::parse::{extract_cache_key, extract_ips, extract_min_ttl};
use crate::monitor::bus::Bus;
use crate::monitor::error_ext::BusErrorExt;
use dashmap::DashMap;
use std::sync::Arc;

pub async fn forward_dns(
    client: Client,
    packet: Vec<u8>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    notify: Arc<Notify>,
    bus: Arc<Bus>,
) -> Result<Vec<u8>> {
    let key = extract_cache_key(&packet)
        .ok_or_else(|| anyhow::anyhow!("failed to extract cache key"))
        .emit_to_bus(&bus)?;
    let ttl = extract_min_ttl(&packet[0..]).unwrap_or(90);
    let ips = extract_ips(&packet[0..]);
    let rc = packet[3] & 0x0F;

    let response = client
        .post("https://cloudflare-dns.com/dns-query")
        .header("content-type", "application/dns-message")
        .header("accept", "application/dns-message")
        .body(packet)
        .send()
        .await
        .emit_to_bus(&bus)?;

    let response_bytes = response.bytes().await.emit_to_bus(&bus)?.to_vec();

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
