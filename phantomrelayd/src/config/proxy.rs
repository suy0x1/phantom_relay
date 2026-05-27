pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
}

impl ProxyConfig {
    pub fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9003,
        }
    }
}
