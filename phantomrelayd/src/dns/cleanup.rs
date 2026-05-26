use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use tokio::time::{Duration, interval};
use tokio::sync::Mutex;

use crate::config::dns::DNSConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::monitor::bus::Bus;
use crate::monitor::events::LifecycleEvent;
use tokio_util::sync::CancellationToken;

pub async fn start_cache_cleanup(
    config: Arc<Mutex<DNSConfig>>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    bus: Arc<Bus>,
    cancel: CancellationToken,
) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(config.lock().await.cache_cleanup_interval_secs));

    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                break;
            }

            _ = ticker.tick() => {
                let now = Instant::now();

                let len_before =
                    cache.len();

                cache.retain(|_, entry| {
                    entry.expires_at > now
                });

                let discarded =
                    len_before - cache.len();

                bus.emit_lifecycle(LifecycleEvent::DNSCacheCleanup {
                    entries_cleaned: discarded,
                    timestamp: SystemTime::now(),
                }).await;
            }
        }
    }

    Ok(())
}
