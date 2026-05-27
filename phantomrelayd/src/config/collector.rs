use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectorConfig {
    pub total_workers: usize,
    pub latency: u64,
}

impl CollectorConfig {
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            total_workers: 100,
            latency: 3500,
        }
    }
}
