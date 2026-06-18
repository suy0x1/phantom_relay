use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use super::context::RuntimeContext;
use crate::config::Config;
use crate::metrics::metrics::Metrics;
use crate::monitor::bus::Bus;
use crate::routing::manager::ConnectionManager;
use crate::runtime::controller::RuntimeController;
use crate::subsystems::rotation::manager::RotationEngine;
use crate::subsystems::rotation::route::RouteContext;
use crate::utils::converter::convert_start;

/// Initializes runtime controller with default configs, metrics, and spawns background managers.
pub async fn startup(bus: Arc<Bus>) -> Result<RuntimeController> {
    let config = Config::load_or_create("./phantomrelay.toml")?;
    let current_route = Arc::new(RwLock::new(RouteContext::dummy()));
    let ctx = RuntimeContext {
        bus: bus.clone(),
        metrics: Arc::new(Metrics::default()),
        rotation_config: Arc::new(config.rotation),
        dns_config: Arc::new(Mutex::new(config.dns)),
        tproxy_config: Arc::new(config.tproxy),
        proxy_config: Arc::new(config.proxy),
        collector_config: Arc::new(Mutex::new(config.collector)),
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

    let mut runtime = RuntimeController::new(ctx, rotation_engine);
    runtime
        .networkmanager
        .clone()
        .spawn_network_manager(runtime.ctx.clone());
    runtime
        .rotation_engine
        .start_rotation_engine(runtime.rotation_engine.clone(), runtime.ctx.clone());

    let default_services = config.default.services.clone();
    for i in default_services {
        runtime.handle_commands(convert_start(&i)?).await?;
    }

    Ok(runtime)
}
