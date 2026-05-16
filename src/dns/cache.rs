use std::time::Instant;

pub struct CacheEntry {
    pub response: Vec<u8>,
    pub expires_at: Instant,
}