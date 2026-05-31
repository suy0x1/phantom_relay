use serde::{Deserialize, Serialize};

/// Configuration for the proxy rotation subsystem.
///
/// Controls how frequently the active proxy is rotated to a different one.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RotationConfig {
    /// Interval in seconds between proxy rotations.
    pub rotate_sec: u64,
}

impl RotationConfig {
    /// Creates a new rotation configuration with default values.
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self { rotate_sec: 60 }
    }
}
