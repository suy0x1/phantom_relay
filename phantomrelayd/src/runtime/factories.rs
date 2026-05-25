use std::sync::Arc;

use super::context::RuntimeContext;
use crate::{
    collector::service::collect_healthy_proxy, dns::{
        cleanup::start_cache_cleanup,
        listener::start_dns_listener,
        prewarmer::{preload::preload_dns_entries, refresh::start_cache_refresh},
    }, metrics::listener::start_metrics, monitor::logger::start_logger, proxy::server::start_socks5_server, subsystems::rotation::service::start_rotating, tproxy::listener::start_listener
};

use super::controller::ServiceFn;

pub fn logger_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move { start_logger(ctx.bus.clone(), cancel).await })
    })
}

pub fn metrics_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move { start_metrics(ctx.bus.clone(), cancel).await })
    })
}

pub fn proxy_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            start_socks5_server(
                ctx.proxy_config.clone(),
                ctx.conn_map.clone(),
                ctx.bus.clone(),
                ctx.current_route.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn dns_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            start_dns_listener(
                ctx.dns_config.clone(),
                ctx.dns_cache.clone(),
                ctx.bus.clone(),
                ctx.inflight.clone(),
                ctx.current_route.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn tproxy_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            start_listener(
                ctx.tproxy_config.clone(),
                ctx.conn_map.clone(),
                ctx.bus.clone(),
                ctx.current_route.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn preload_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            preload_dns_entries(
                ctx.dns_config.clone(),
                ctx.bus.clone(),
                ctx.dns_cache.clone(),
                ctx.inflight.clone(),
                ctx.current_route.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn refresh_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            start_cache_refresh(
                ctx.dns_config.clone(),
                ctx.bus.clone(),
                ctx.dns_cache.clone(),
                ctx.inflight.clone(),
                ctx.current_route.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn cleanup_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            start_cache_cleanup(
                ctx.dns_config.clone(),
                ctx.dns_cache.clone(),
                ctx.bus.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn collector_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            collect_healthy_proxy(
                ctx.collector_config.clone(),
                ctx.bus.clone(),
                ctx.healthy_proxies.clone(),
                cancel,
            )
            .await
        })
    })
}

pub fn rotator_service(ctx: Arc<RuntimeContext>) -> ServiceFn {
    Arc::new(move |cancel| {
        let ctx = ctx.clone();

        Box::pin(async move {
            start_rotating(
                ctx.rotation_config.clone(),
                ctx.bus.clone(),
                ctx.healthy_proxies.clone(),
                cancel,
            )
            .await
        })
    })
}
