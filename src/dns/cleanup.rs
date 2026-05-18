use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{Duration, interval};
use anyhow::Result;

use crate::config::dns::DNSConfig;
use crate::dns::cache::{CacheKey, CacheEntry};
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{DNSCacheCleanup};
use chrono::Local;

pub async fn start_cache_cleanup(config: Arc<DNSConfig>, cache: Arc<DashMap<CacheKey, CacheEntry>> , bus: Arc<Bus>) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(config.cache_cleanup_interval_secs));

    loop {
        ticker.tick().await;
        let now = Instant::now();
        let len_before = cache.len();
        cache.retain(|_,entry| {
            entry.expires_at > now
        });
        let discarded = len_before - cache.len();
        bus.emit(DNSCacheCleanup { entries_cleaned: discarded, timestamp: Local::now().format("%H:%M:%S").to_string()})?;
    }
}