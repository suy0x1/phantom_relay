use serde::{Deserialize, Serialize};

/// Configuration for the transparent proxy subsystem.
///
/// Specifies the network address and port where the transparent proxy listener binds.
/// This is used for intercepting outgoing traffic via netfilter rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TProxyConfig {
    /// Host address where the transparent proxy listener binds (e.g., "127.0.0.1").
    pub host: String,
    /// Port where the transparent proxy listener accepts intercepted connections.
    pub port: u16,
}

impl TProxyConfig {
    /// Creates a new transparent proxy configuration with default values.
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
