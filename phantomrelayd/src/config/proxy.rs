use serde::{Deserialize, Serialize};

/// Configuration for the SOCKS5 proxy service.
///
/// Specifies the network address and port where the SOCKS5 proxy listens for client connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    /// Host address where the SOCKS5 proxy binds (e.g., "127.0.0.1").
    pub host: String,
    /// Port where the SOCKS5 proxy accepts client connections.
    pub port: u16,
}

impl ProxyConfig {
    /// Creates a new proxy configuration with default values.
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
