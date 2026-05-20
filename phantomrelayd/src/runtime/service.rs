use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct ServiceHandle {
    pub task: JoinHandle<()>,
    pub cancel: CancellationToken,
}

pub enum Service {
    Logger,
    DNS,
    CacheReloader,
    CacheCleaner,
    CachePreloader,
    CacheRefresher,
    TProxy,
    Proxy,
    Metrics,
}

