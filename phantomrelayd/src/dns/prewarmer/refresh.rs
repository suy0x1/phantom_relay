use crate::config::dns::DNSConfig;
use crate::dns::parse::extract_cache_key;
use crate::dns::prewarmer::packet::build_dns_query;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{TaskShutdown, TaskStartup};

use crate::dns::cache::{CacheEntry, CacheKey};
use crate::subsystems::rotation::route::RouteContext;
use anyhow::Result;
use chrono::Local;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::sync::{Mutex, Notify};
use tokio::time::{Duration, interval};
use tokio_util::sync::CancellationToken;

pub async fn start_cache_refresh(
    config: Arc<Mutex<DNSConfig>>,
    bus: Arc<Bus>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    current: Arc<RwLock<RouteContext>>,
    cancel: CancellationToken,
) -> Result<()> {
    let _ = bus.emit(TaskStartup {
        task_name: "DNS Cache Refresher".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    });
    let (cache_ref_sec, cs, mph, pd) = {
        let cfg = config.lock().await;
        (
            cfg.cache_refresh_secs,
            cfg.cache_saturation,
            cfg.min_prest_hits,
            cfg.prewarm_domains.clone(),
        )
    };

    let mut ticker = interval(Duration::from_secs(cache_ref_sec));

    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                let _ = bus.emit(TaskShutdown {
                    task_name: "DNS Cache Refresher".to_string(),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                });
                break;
            }

            _ = ticker.tick() => {

                let now = Instant::now();

                for entry in cache.iter() {

                    if cancel.is_cancelled() {
                        break;
                    }

                    let key =
                        entry.key().clone();

                    let hits =
                        entry.hits.load(
                            Ordering::Relaxed
                        ) as u64;

                    let rcode =
                        entry.rcode.clone();

                    let domain =
                        extract_cache_key(
                            &entry.response[0..]
                        )
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "failed to extract cache key"
                            )
                        })?
                        .domain;

                    let remaining =
                        entry.expires_at
                            .saturating_duration_since(now);

                    if remaining >
                        Duration::from_secs(15)
                    {
                        continue;
                    }

                    if inflight.contains_key(&key) {
                        continue;
                    }

                    if !(cs) {
                        if hits <
                            mph
                            &&
                            !pd
                                .contains(&domain)
                        {
                            continue;
                        }
                    }

                    if matches!(rcode, 2 | 3) {
                        continue;
                    }

                    let notify =
                        Arc::new(Notify::new());

                    inflight.insert(
                        key.clone(),
                        notify.clone(),
                    );

                    let cache =
                        cache.clone();

                    let inflight =
                        inflight.clone();
                    let client = current.read().await.clone().client;
                    tokio::spawn(async move {

                        let packet =
                            build_dns_query(
                                &key.domain,
                                key.qtype,
                            );
                            
                        let result =
                            crate::dns::doh::forward_dns(
                                client,
                                packet,
                                cache,
                                inflight.clone(),
                                notify.clone(),
                            )
                            .await;

                        if result.is_err() {
                            inflight.remove(&key);

                            notify.notify_waiters();
                        }
                    });
                }
            }
        }
    }

    Ok(())
}
