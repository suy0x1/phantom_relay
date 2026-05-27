use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use super::context::RuntimeContext;
use crate::config::collector::CollectorConfig;
use crate::config::dns::DNSConfig;
use crate::config::proxy::ProxyConfig;
use crate::config::rotation::RotationConfig;
use crate::config::tproxy::TProxyConfig;
use crate::metrics::metrics::Metrics;
use crate::monitor::bus::Bus;
use crate::routing::manager::ConnectionManager;
use crate::runtime::controller::RuntimeController;
use crate::subsystems::rotation::manager::RotationEngine;
use crate::subsystems::rotation::route::RouteContext;

/// Initializes runtime controller with default configs, metrics, and spawns background managers.
pub async fn startup(bus: Arc<Bus>) -> Result<RuntimeController> {
    let current_route = Arc::new(RwLock::new(RouteContext::dummy()));
    let ctx = RuntimeContext {
        bus: bus.clone(),
        metrics: Arc::new(Metrics::default()),
        rotation_config: Arc::new(RotationConfig::default()),
        dns_config: Arc::new(Mutex::new(DNSConfig::default())),
        tproxy_config: Arc::new(TProxyConfig::default()),
        proxy_config: Arc::new(ProxyConfig::default()),
        collector_config: Arc::new(Mutex::new(CollectorConfig::default())),
        current_route: current_route.clone(),
        conn_map: Arc::new(ConnectionManager::new()),
        dns_cache: Arc::new(DashMap::new()),
        inflight: Arc::new(DashMap::new()),
        healthy_proxies: Arc::new(DashMap::new()),
    };

    let rotation_engine = Arc::new(RotationEngine {
        current: current_route,
        cursor: AtomicUsize::new(0),
    });

    let runtime = RuntimeController::new(ctx, rotation_engine);
    runtime
        .networkmanager
        .clone()
        .spawn_network_manager(runtime.ctx.clone());
    runtime
        .rotation_engine
        .start_rotation_engine(runtime.rotation_engine.clone(), runtime.ctx.clone());

    Ok(runtime)
}
