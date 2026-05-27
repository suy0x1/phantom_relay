pub struct DNSConfig {
    pub host: String,
    pub port: u16,
    pub max_parallel_dns_lookups: usize,
    pub cache_cleanup_interval_secs: u64,
    pub cache_refresh_secs: u64,
    pub min_prest_hits: u64,
    pub cache_saturation: bool,
    pub prewarm_domains: Vec<String>,
}

impl DNSConfig {
    pub fn default() -> Self {
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
