use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Service {
    Logger,
    ProxyCollector,
    DNS,
    ProxyRotator,
    CacheCleaner,
    CachePreloader,
    CacheRefresher,
    TProxy,
    Proxy,
    Metrics,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Mode {
    CacheReloader,
}
