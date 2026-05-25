use crate::monitor::events::Event::{LoadInitialProxy, RotateProxy};
use crate::{
    collector::manager::HealthyProxy, runtime::context::RuntimeContext,
    subsystems::rotation::route::RouteContext,
};
use anyhow::Result;
use dashmap::DashMap;
use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;

pub struct RotationEngine {
    pub current: Arc<RwLock<RouteContext>>,
    pub cursor: AtomicUsize,
}

impl RotationEngine {
    pub async fn rotate(&self, proxies: Arc<DashMap<HealthyProxy, Client>>) -> Result<()> {
        let entries: Vec<_> = proxies.iter().collect();

        if entries.is_empty() {
            return Err(anyhow::anyhow!(
                "rotation attempted with empty healthy proxy pool"
            ));
        }

        let idx = self.cursor.fetch_add(1, Ordering::Relaxed);

        let proxy = &entries[idx % entries.len()];

        let ctx = RouteContext {
            proxy: proxy.key().clone(),
            client: proxy.value().clone(),
        };

        let mut data = self.current.write().await;

        *data = ctx;

        Ok(())
    }
    pub fn start_rotation_engine(&self, engine: Arc<RotationEngine>, ctx: Arc<RuntimeContext>) {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            loop {
                let mut rx = ctx.bus.subscribe();
                _ = match rx.recv().await {
                    Ok(LoadInitialProxy { timestamp: _ }) => {
                        engine.rotate(ctx.healthy_proxies.clone()).await
                    }
                    Ok(RotateProxy { timestamp: _ }) => {
                        engine.rotate(ctx.healthy_proxies.clone()).await
                    }
                    _ => Ok(()),
                };
            }
        });
    }
}
