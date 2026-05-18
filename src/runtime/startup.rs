use anyhow::Result;
use chrono::Local;
use dashmap::DashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Notify;

use crate::dns::cache::{CacheEntry, CacheKey};
use crate::dns::cleanup::start_cache_cleanup;
use crate::dns::listener::start_dns_listener;
use crate::metrics::listener::start_metrics;
use crate::dns::prewarmer::refresh::start_cache_refresh;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::Error;
use crate::monitor::logger::start_logger;
use crate::redirect::listener::start_listener;
use crate::relay::manager::ConnectionManager;
use crate::system::nftables::network_setup;
use crate::dns::prewarmer::preload::preload_dns_entries;
use crate::config::dns::DNSConfig;

fn spawn_task<F>(bus: Arc<Bus>, fut: F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(e) = fut.await {
            let _ = bus.emit(Error {
                err: e.to_string(),
                timestamp: Local::now().format("%H:%M:%S").to_string(),
            });
        }
    });
}

pub async fn startup(proxy_port: u16, dns_port: u16, bus: Arc<Bus>) -> Result<()> {
    let dns_config = Arc::new(DNSConfig::default());

    let conn_map = Arc::new(ConnectionManager::new());

    let dns_cache: Arc<DashMap<CacheKey, CacheEntry>> = Arc::new(DashMap::new());

    let inflight: Arc<DashMap<CacheKey, Arc<Notify>>> = Arc::new(DashMap::new());

    spawn_task(bus.clone(), start_logger(bus.clone()));

    spawn_task(bus.clone(), start_metrics(bus.clone()));
    
    spawn_task(bus.clone(),start_dns_listener(dns_config.clone(),dns_cache.clone(), bus.clone(), inflight.clone()));
    
    spawn_task(bus.clone(), start_listener(conn_map.clone(), bus.clone()));
    
    spawn_task(bus.clone(), preload_dns_entries(dns_config.clone(), bus.clone(), dns_cache.clone(), inflight.clone()));
    
    spawn_task(bus.clone(), start_cache_refresh(dns_config.clone(), bus.clone(), dns_cache.clone(), inflight.clone()));
    
    spawn_task(bus.clone(),start_cache_cleanup(dns_config.clone(), dns_cache.clone(), bus.clone()));
    
    network_setup(proxy_port, dns_port, bus.clone())?;

    Ok(())
}
