pub struct CollectorConfig {
    pub total_workers: usize,
    pub latency: u128,

}

impl CollectorConfig {
    pub fn default() -> Self {
        Self {
            total_workers: 100,
            latency: 3500,
        }
    }
}