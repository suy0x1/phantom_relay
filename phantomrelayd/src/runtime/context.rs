use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{Notify, Mutex};

use crate::config::dns::DNSConfig;
use crate::config::proxy::ProxyConfig;
use crate::config::tproxy::TProxyConfig;
use crate::dns::cache::{CacheEntry, CacheKey};
use crate::monitor::bus::Bus;
use crate::routing::manager::ConnectionManager;

pub struct RuntimeContext {
    pub bus: Arc<Bus>,
    pub dns_config: Arc<Mutex<DNSConfig>>,
    pub tproxy_config: Arc<TProxyConfig>,
    pub proxy_config: Arc<ProxyConfig>,
    pub conn_map: Arc<ConnectionManager>,
    pub dns_cache: Arc<DashMap<CacheKey, CacheEntry>>,
    pub inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
}
