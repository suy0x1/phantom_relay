use std::net::IpvAddr;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct CacheKey {
    pub domain: String,
    pub qtype: u16,
    pub qclass: u16,
}
pub struct CacheEntry {
    pub response: Vec<u8>,
    pub expires_at: Instant,
    pub resolved_ips: Vec<IpvAddr>,
    pub hits: AtomicU64,
}

pub struct Metrics {
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub upstream_requests: AtomicU64,
    pub inflight_waits: AtomicU64,
}
