use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

use crate::config::{
    collector::CollectorConfig, dns::DNSConfig, proxy::ProxyConfig, rotation::RotationConfig,
    tproxy::TProxyConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    rotation_config: RotationConfig,
    dns_config: DNSConfig,
    tproxy_config: TProxyConfig,
    proxy_config: ProxyConfig,
    collector_config: CollectorConfig,
}

impl DebugConfig {
    pub async fn from_state(
        rotation_config: &Arc<RotationConfig>,
        dns_config: &Arc<Mutex<DNSConfig>>,
        tproxy_config: &Arc<TProxyConfig>,
        proxy_config: &Arc<ProxyConfig>,
        collector_config: &Arc<Mutex<CollectorConfig>>,
    ) -> Self {
        Self {
            rotation_config: rotation_config.as_ref().clone(),
            dns_config: dns_config.lock().await.clone(),
            tproxy_config: tproxy_config.as_ref().clone(),
            proxy_config: proxy_config.as_ref().clone(),
            collector_config: collector_config.lock().await.clone(),
        }
    }
}

pub fn debug_config(debug: DebugConfig) -> Result<String> {
    Ok(format!("{:#?}", debug))
}
