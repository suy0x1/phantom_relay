use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct ServiceHandle {
    pub task: JoinHandle<()>,
    pub cancel: CancellationToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Service {
    Logger,
    ProxyCollector,
    DNS,
    CacheCleaner,
    CachePreloader,
    CacheRefresher,
    TProxy,
    Proxy,
    Metrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mode {
    CacheReloader,
}