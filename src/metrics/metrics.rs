#[derive(Default)]
pub struct Metrics {
    pub dns_requests: u64,

    pub dns_cache_hits: u64,
    pub dns_cache_misses: u64,

    pub proxy_connected: u64,
    pub proxy_failed: u64,

    pub connections_opened: u64,
    pub connections_closed: u64,

    pub errors: u64,
}