use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Notify;

use crate::config::proxy::ProxyConfig;
use crate::config::tproxy::TProxyConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::monitor::bus::Bus;
use crate::runtime::controller::RuntimeController;
use crate::routing::manager::ConnectionManager;
use crate::runtime::factories::{cleanup_service, dns_service, logger_service, metrics_service, preload_service, proxy_service, refresh_service, tproxy_service};
use crate::system::nftables::network_setup;
use crate::config::dns::DNSConfig;


pub async fn startup(bus: Arc<Bus>) -> Result<RuntimeController> {

    let dns_config = Arc::new(DNSConfig::default());
    let tproxy_config = Arc::new(TProxyConfig::default());
    let proxy_config = Arc::new(ProxyConfig::defualt());
    let conn_map = Arc::new(ConnectionManager::new());
    let dns_cache: Arc<DashMap<CacheKey, CacheEntry>> = Arc::new(DashMap::new());
    let inflight: Arc<DashMap<CacheKey, Arc<Notify>>> = Arc::new(DashMap::new());
    
    let runtime = RuntimeController::new();

    runtime.start_service("logger", logger_service(bus.clone()))?;
    runtime.start_service("metrics", metrics_service(bus.clone()))?;
    runtime.start_service("dns", dns_service(dns_config.clone(),dns_cache.clone(), bus.clone(),inflight.clone()))?;
    runtime.start_service("tproxy", tproxy_service(tproxy_config.clone(), conn_map.clone(), bus.clone()))?;
    runtime.start_service("proxy", proxy_service(proxy_config.clone(), conn_map.clone(), bus.clone()))?;
    runtime.start_service("refresher", refresh_service(dns_config.clone(), bus.clone(), dns_cache.clone(), inflight.clone()))?;
    runtime.start_service("preloader", preload_service(dns_config.clone(), bus.clone(), dns_cache.clone(), inflight.clone()))?;
    runtime.start_service("cleaner", cleanup_service(dns_config.clone(), dns_cache.clone(), bus.clone()))?;


    network_setup(tproxy_config.port, dns_config.port, bus.clone())?;

    Ok(runtime)
}
