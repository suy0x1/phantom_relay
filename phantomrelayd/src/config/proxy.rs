use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
}

impl ProxyConfig {
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9003,
        }
    }
}
