use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tokio::sync::Mutex;

use super::context::RuntimeContext;
use crate::config::collector::CollectorConfig;
use crate::config::dns::DNSConfig;
use crate::config::proxy::ProxyConfig;
use crate::config::tproxy::TProxyConfig;
use crate::monitor::bus::Bus;
use crate::routing::manager::ConnectionManager;
use crate::runtime::controller::RuntimeController;


pub async fn startup(bus: Arc<Bus>) -> Result<RuntimeController> {
    let ctx = RuntimeContext {
        bus: bus.clone(),
        dns_config: Arc::new(Mutex::new(DNSConfig::default())),
        tproxy_config: Arc::new(TProxyConfig::default()),
        proxy_config: Arc::new(ProxyConfig::default()),
        collector_config: Arc::new(Mutex::new(CollectorConfig::default())),
        conn_map: Arc::new(ConnectionManager::new()),
        dns_cache: Arc::new(DashMap::new()),
        inflight: Arc::new(DashMap::new()),
        healthy_proxies: Arc::new(DashMap::new()),
    };

    let runtime = RuntimeController::new(ctx);
    let network_cancel = CancellationToken::new();
    runtime.networkmanager.clone().spawn_network_manager(runtime.ctx.clone(), network_cancel.clone());
    
    Ok(runtime)
}
