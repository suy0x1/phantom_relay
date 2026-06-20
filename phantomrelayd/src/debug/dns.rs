use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::Notify;

use crate::dns::cache::{CacheEntry, CacheKey};

pub struct DebugDns {
    cache: Arc<DashMap<CacheKey, CacheEntry>>,
    inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
}

impl DebugDns {
    pub fn new(
        cache: Arc<DashMap<CacheKey, CacheEntry>>,
        inflight: Arc<DashMap<CacheKey, Arc<Notify>>>,
    ) -> Self {
        Self {
            cache: cache,
            inflight: inflight,
        }
    }
}

pub fn debug_dns(debug: DebugDns) -> Result<String> {
    let data = format!("{:#?} {:#?}", debug.cache, debug.inflight);
    Ok(data)
}
