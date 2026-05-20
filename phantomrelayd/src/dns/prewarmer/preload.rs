use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tokio::sync::Mutex;

use crate::config::dns::DNSConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::doh::forward_dns;
use crate::dns::prewarmer::packet::build_dns_query;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::{Info, TaskStartup, TaskShutdown};
use chrono::Local;

pub async fn preload_dns_entries(
    config: Arc<Mutex<DNSConfig>>,
    bus: Arc<Bus>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    cancel: CancellationToken,
) -> Result<()> {
    let _ = bus.emit(TaskStartup {
        task_name: "DNS Cache Preloader".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    });

    for domain in &config.lock().await.prewarm_domains {
        if cancel.is_cancelled() {
            let _ = bus.emit(TaskShutdown {
                task_name: "DNS Cache Preloader".to_string(),
                timestamp: Local::now().format("%H:%M:%S").to_string(),
            });
            break;
        }

        // A record
        let a_packet = build_dns_query(domain, 1);

        let a_notify = Arc::new(Notify::new());

        let a_key = CacheKey {
            domain: domain.clone(),
            qtype: 1,
            qclass: 1,
        };

        inflight.insert(a_key, a_notify.clone());

        let _ = forward_dns(a_packet, cache.clone(), inflight.clone(), a_notify).await;

        if cancel.is_cancelled() {
            break;
        }

        // AAAA record
        let aaaa_packet = build_dns_query(domain, 28);

        let aaaa_notify = Arc::new(Notify::new());

        let aaaa_key = CacheKey {
            domain: domain.clone(),
            qtype: 28,
            qclass: 1,
        };

        inflight.insert(aaaa_key, aaaa_notify.clone());

        let _ = forward_dns(aaaa_packet, cache.clone(), inflight.clone(), aaaa_notify).await;
    }

    let _ = bus.emit(Info {
        content: format!("preloaded {} cache entires", cache.len()),

        timestamp: Local::now().format("%H:%M:%S").to_string(),
    });

    Ok(())
}
