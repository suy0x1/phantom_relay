use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::Notify;

use crate::{
    config::{dns::DNSConfig, proxy::ProxyConfig, tproxy::TProxyConfig},
    dns::{
        cache::{CacheEntry, CacheKey},
        cleanup::start_cache_cleanup,
        listener::start_dns_listener,
        prewarmer::{preload::preload_dns_entries, refresh::start_cache_refresh},
    },
    metrics::listener::start_metrics,
    monitor::{bus::Bus, logger::start_logger},
    proxy::server::start_socks5_server,
    routing::manager::ConnectionManager,
    tproxy::listener::start_listener,
};

use super::controller::ServiceFn;

pub fn logger_service(bus: Arc<Bus>) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        Box::pin(async move { start_logger(bus, cancel).await })
    })
}

pub fn metrics_service(bus: Arc<Bus>) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        Box::pin(async move { start_metrics(bus, cancel).await })
    })
}

pub fn proxy_service(
    config: Arc<ProxyConfig>,
    conn_map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        let config = config.clone();
        let conn_map = conn_map.clone();
        Box::pin(async move { start_socks5_server(config, conn_map, bus, cancel).await })
    })
}

pub fn dns_service(
    config: Arc<DNSConfig>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    bus: Arc<Bus>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        let config = config.clone();
        let cache = cache.clone();
        let inflight = inflight.clone();
        Box::pin(async move { start_dns_listener(config, cache, bus, inflight, cancel).await })
    })
}

pub fn tproxy_service(
    config: Arc<TProxyConfig>,
    conn_map: Arc<ConnectionManager>,
    bus: Arc<Bus>,
) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        let config = config.clone();
        let conn_map = conn_map.clone();
        Box::pin(async move { start_listener(config, conn_map, bus, cancel).await })
    })
}

pub fn preload_service(
    config: Arc<DNSConfig>,
    bus: Arc<Bus>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        let config = config.clone();
        let cache = cache.clone();
        let inflight = inflight.clone();
        Box::pin(async move { preload_dns_entries(config, bus, cache, inflight, cancel).await })
    })
}

pub fn refresh_service(
    config: Arc<DNSConfig>,
    bus: Arc<Bus>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        let config = config.clone();
        let cache = cache.clone();
        let inflight = inflight.clone();
        Box::pin(async move { start_cache_refresh(config, bus, cache, inflight, cancel).await })
    })
}

pub fn cleanup_service(
    config: Arc<DNSConfig>,
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    bus: Arc<Bus>,
) -> ServiceFn {
    Arc::new(move |cancel| {
        let bus = bus.clone();
        let config = config.clone();
        let cache = cache.clone();
        Box::pin(async move { start_cache_cleanup(config, cache, bus, cancel).await })
    })
}
