use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TProxyConfig {
    pub host: String,
    pub port: u16,
}

impl TProxyConfig {
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for TProxyConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9001,
        }
    }
}
