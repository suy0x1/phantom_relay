use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RotationConfig {
    pub rotate_sec: u64,
}

impl RotationConfig {
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self { rotate_sec: 60 }
    }
}
