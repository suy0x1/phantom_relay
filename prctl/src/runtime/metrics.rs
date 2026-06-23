use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MetricsSnapshot {
    pub service_startups: u64,
    pub service_shutdowns: u64,

    pub network_changes: u64,

    pub connections_opened: u64,
    pub connections_closed: u64,

    pub dns_requests: u64,
    pub dns_cache_hits: u64,
    pub dns_cache_misses: u64,

    pub proxy_connected: u64,
    pub proxy_failed: u64,

    pub task_startups: u64,
    pub task_shutdowns: u64,

    pub capability_enabled: u64,
    pub capability_disabled: u64,

    pub load_initial_proxy: u64,
    pub rotate_proxy: u64,
    pub routing_decisions: u64,

    pub info_events: u64,
    pub errors: u64,

    pub dns_cache_cleanup: u64,

    pub lagged_critical: u64,
    pub lagged_telemetry: u64,
    pub lagged_lifecycle: u64,
    pub lagged_diagnostic: u64,
}