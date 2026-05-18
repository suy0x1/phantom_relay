use std::net::IpAddr;
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
    pub resolved_ips: Vec<IpAddr>,
    pub hits: AtomicU64,
    pub rcode: u8,
}
