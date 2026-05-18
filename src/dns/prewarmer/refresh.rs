use crate::config::dns::DNSConfig;
use crate::dns::prewarmer::packet::build_dns_query;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::TaskStartup;

use crate::dns::cache::{CacheEntry, CacheKey};
use anyhow::Result;
use chrono::Local;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio::sync::Notify;
use tokio::time::{Duration, interval};

pub async fn start_cache_refresh(
    config: Arc<DNSConfig>,
    bus: Arc<Bus>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
) -> Result<()> {
    let _ = bus.emit(TaskStartup {
        task_name: "DNS Cache Saturator".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    });
    let mut ticker = interval(Duration::from_secs(config.cache_refresh_secs));

    loop {
        ticker.tick().await;

        let now = Instant::now();

        for entry in cache.iter() {
            let key = entry.key().clone();
            let hits = entry.hits.load(Ordering::Relaxed) as u64;
            let rcode = entry.rcode.clone();

            let remaining = entry.expires_at.saturating_duration_since(now);

            if remaining > Duration::from_secs(15) {
                continue;
            }

            if inflight.contains_key(&key) {
                continue;
            }

            if !(config.cache_saturation) {
                if hits < config.min_prest_hits {
                    continue;
                }
            } 

            if matches!(rcode, 2|3) {
                continue;
            }

            let notify = Arc::new(Notify::new());

            inflight.insert(key.clone(), notify.clone());

            let cache = cache.clone();
            let inflight = inflight.clone();

            tokio::spawn(async move {
                let packet = build_dns_query(&key.domain, key.qtype);

                let result =
                    crate::dns::doh::forward_dns(packet, cache, inflight.clone(), notify.clone())
                        .await;

                if result.is_err() {
                    inflight.remove(&key);
                    notify.notify_waiters();
                }
            });
        }
    }
}
