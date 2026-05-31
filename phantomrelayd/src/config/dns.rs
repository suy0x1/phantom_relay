use serde::{Deserialize, Serialize};

/// Configuration for the DNS subsystem.
///
/// Controls DNS listener settings, caching behavior, and DNS prewarming (cache preloading).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DNSConfig {
    /// Host address where the DNS listener binds (e.g., "127.0.0.1").
    pub host: String,
    /// Port where the DNS listener accepts queries.
    pub port: u16,
    /// Maximum number of concurrent DNS lookups allowed.
    pub max_parallel_dns_lookups: usize,
    /// Interval in seconds for removing stale entries from the DNS cache.
    pub cache_cleanup_interval_secs: u64,
    /// Interval in seconds for refreshing cached DNS entries.
    pub cache_refresh_secs: u64,
    /// Minimum number of cache hits before a domain is considered "prewarmed".
    pub min_prest_hits: u64,
    /// Whether to enable DNS cache saturation mode (prewarming all configured domains).
    pub cache_saturation: bool,
    /// List of domain names to preload into the DNS cache on startup.
    pub prewarm_domains: Vec<String>,
}

impl DNSConfig {
    /// Creates a new DNS configuration with default values.
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for DNSConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9002,
            max_parallel_dns_lookups: 100,
            cache_cleanup_interval_secs: 30,
            cache_refresh_secs: 5,
            min_prest_hits: 25,
            cache_saturation: false,
            prewarm_domains: vec![
                "google.com".to_string(),
                "chatgpt.com".to_string(),
                "youtube.com".to_string(),
                "github.com".to_string(),
                "ping.archlinux.org".to_string(),
                "www.discord.com".to_string(),
                "go-updater.brave.com".to_string(),
                "mobile.events.data.microsoft.com".to_string(),
                "main.vscode-cdn.net".to_string(),
                "cdn.discordapp.com".to_string(),
            ],
        }
    }
}
