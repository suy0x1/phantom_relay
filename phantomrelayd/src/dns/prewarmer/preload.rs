use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::config::dns::DNSConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::doh::forward_dns;
use crate::dns::prewarmer::packet::build_dns_query;
use crate::monitor::bus::Bus;
use crate::monitor::events::{DiagnosticEvent, LifecycleEvent};
use crate::subsystems::rotation::route::RouteContext;
use tokio::sync::RwLock;

/// Pre-resolves configured domains (A and AAAA records) into cache before queries arrive.
pub async fn preload_dns_entries(
    config: Arc<Mutex<DNSConfig>>,
    bus: Arc<Bus>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    current: Arc<RwLock<RouteContext>>,
    cancel: CancellationToken,
) -> Result<()> {
    _ = bus.emit_lifecycle(LifecycleEvent::TaskStartup {
        task_name: "DNS Cache Preloader".to_string(),
        timestamp: SystemTime::now(),
    });

    for domain in &config.lock().await.prewarm_domains {
        if cancel.is_cancelled() {
            _ = bus.emit_lifecycle(LifecycleEvent::TaskShutdown {
                task_name: "DNS Cache Preloader".to_string(),
                timestamp: SystemTime::now(),
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
        let client = current.read().await.clone().client;
        let _ = forward_dns(
            client.clone(),
            a_packet,
            cache.clone(),
            inflight.clone(),
            a_notify,
            bus.clone(),
        )
        .await;

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

        let _ = forward_dns(
            client,
            aaaa_packet,
            cache.clone(),
            inflight.clone(),
            aaaa_notify,
            bus.clone(),
        )
        .await;
    }

    _ = bus.emit_diagnostic(DiagnosticEvent::Info {
        content: format!("preloaded {} cache entires", cache.len()),
        timestamp: SystemTime::now(),
    });

    Ok(())
}
