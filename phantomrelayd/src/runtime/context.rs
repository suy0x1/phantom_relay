use dashmap::DashMap;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

use crate::collector::manager::HealthyProxy;
use crate::config::collector::CollectorConfig;
use crate::config::dns::DNSConfig;
use crate::config::proxy::ProxyConfig;
use crate::config::rotation::RotationConfig;
use crate::config::tproxy::TProxyConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::metrics::metrics::Metrics;
use crate::monitor::bus::Bus;
use crate::routing::manager::ConnectionManager;
use crate::subsystems::rotation::route::RouteContext;
use tokio::sync::RwLock;

pub struct RuntimeContext {
    pub bus: Arc<Bus>,
    pub metrics: Arc<Metrics>,
    pub rotation_config: Arc<RotationConfig>,
    pub dns_config: Arc<Mutex<DNSConfig>>,
    pub tproxy_config: Arc<TProxyConfig>,
    pub proxy_config: Arc<ProxyConfig>,
    pub collector_config: Arc<Mutex<CollectorConfig>>,
    pub current_route: Arc<RwLock<RouteContext>>,
    pub conn_map: Arc<ConnectionManager>,
    pub dns_cache: Arc<DashMap<CacheKey, CacheEntry>>,
    pub inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    pub healthy_proxies: Arc<DashMap<HealthyProxy, Client>>,
}
