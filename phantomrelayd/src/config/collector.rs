use serde::{Deserialize, Serialize};

/// Configuration for the proxy collector subsystem.
///
/// Controls how many concurrent workers fetch proxies and how frequently they are collected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectorConfig {
    /// Number of concurrent worker tasks fetching proxies from public sources.
    pub total_workers: usize,
    /// Delay in milliseconds between proxy collection cycles.
    pub latency: u64,
}

impl CollectorConfig {
    /// Creates a new collector configuration with default values.
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
