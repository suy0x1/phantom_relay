use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_utils::CachePadded;

/// Thread-safe counters for system metrics.
///
/// Uses cache-padded atomic operations to minimize contention in a multi-threaded environment.
/// Each counter tracks a specific event type across the relay system.
#[derive(Debug)]
pub struct Metrics {
    /// Number of services that have started.
    pub service_startups: CachePadded<AtomicU64>,
    /// Number of services that have shut down.
    pub service_shutdowns: CachePadded<AtomicU64>,

    /// Number of network configuration changes detected.
    pub network_changes: CachePadded<AtomicU64>,

    /// Number of client connections opened.
    pub connections_opened: CachePadded<AtomicU64>,
    /// Number of client connections closed.
    pub connections_closed: CachePadded<AtomicU64>,

    /// Number of DNS queries processed.
    pub dns_requests: CachePadded<AtomicU64>,
    /// Number of DNS queries answered from cache.
    pub dns_cache_hits: CachePadded<AtomicU64>,
    /// Number of DNS queries not found in cache.
    pub dns_cache_misses: CachePadded<AtomicU64>,

    /// Number of successful proxy connections.
    pub proxy_connected: CachePadded<AtomicU64>,
    /// Number of failed proxy connection attempts.
    pub proxy_failed: CachePadded<AtomicU64>,

    /// Number of background tasks that have started.
    pub task_startups: CachePadded<AtomicU64>,
    /// Number of background tasks that have shut down.
    pub task_shutdowns: CachePadded<AtomicU64>,

    /// Number of times network capabilities were enabled.
    pub capability_enabled: CachePadded<AtomicU64>,
    /// Number of times network capabilities were disabled.
    pub capability_disabled: CachePadded<AtomicU64>,

    /// Number of times the initial proxy was loaded.
    pub load_initial_proxy: CachePadded<AtomicU64>,
    /// Number of proxy rotations that occurred.
    pub rotate_proxy: CachePadded<AtomicU64>,
    /// Number of routing decisions made.
    pub routing_decisions: CachePadded<AtomicU64>,

    /// Number of informational diagnostic events.
    pub info_events: CachePadded<AtomicU64>,
    /// Number of error events.
    pub errors: CachePadded<AtomicU64>,

    /// Total number of DNS cache entries cleaned up.
    pub dns_cache_cleanup: CachePadded<AtomicU64>,

    /// Number of dropped critical events due to channel lag.
    pub lagged_critical: CachePadded<AtomicU64>,
    /// Number of dropped telemetry events due to channel lag.
    pub lagged_telemetry: CachePadded<AtomicU64>,
    /// Number of dropped lifecycle events due to channel lag.
    pub lagged_lifecycle: CachePadded<AtomicU64>,
    /// Number of dropped diagnostic events due to channel lag.
    pub lagged_diagnostic: CachePadded<AtomicU64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            service_startups: CachePadded::new(AtomicU64::new(0)),
            service_shutdowns: CachePadded::new(AtomicU64::new(0)),

            network_changes: CachePadded::new(AtomicU64::new(0)),

            connections_opened: CachePadded::new(AtomicU64::new(0)),
            connections_closed: CachePadded::new(AtomicU64::new(0)),

            dns_requests: CachePadded::new(AtomicU64::new(0)),
            dns_cache_hits: CachePadded::new(AtomicU64::new(0)),
            dns_cache_misses: CachePadded::new(AtomicU64::new(0)),

            proxy_connected: CachePadded::new(AtomicU64::new(0)),
            proxy_failed: CachePadded::new(AtomicU64::new(0)),

            task_startups: CachePadded::new(AtomicU64::new(0)),
            task_shutdowns: CachePadded::new(AtomicU64::new(0)),

            capability_enabled: CachePadded::new(AtomicU64::new(0)),
            capability_disabled: CachePadded::new(AtomicU64::new(0)),

            load_initial_proxy: CachePadded::new(AtomicU64::new(0)),
            rotate_proxy: CachePadded::new(AtomicU64::new(0)),
            routing_decisions: CachePadded::new(AtomicU64::new(0)),

            info_events: CachePadded::new(AtomicU64::new(0)),
            errors: CachePadded::new(AtomicU64::new(0)),

            dns_cache_cleanup: CachePadded::new(AtomicU64::new(0)),

            lagged_critical: CachePadded::new(AtomicU64::new(0)),
            lagged_telemetry: CachePadded::new(AtomicU64::new(0)),
            lagged_lifecycle: CachePadded::new(AtomicU64::new(0)),
            lagged_diagnostic: CachePadded::new(AtomicU64::new(0)),
        }
    }
}

/// A snapshot of metrics at a point in time.
///
/// Captures the state of all counters. Suitable for serialization and reporting.
#[derive(Debug, Clone, Copy, Default)]
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

impl Metrics {
    #[inline]
    fn bump(c: &CachePadded<AtomicU64>) {
        c.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn add(c: &CachePadded<AtomicU64>, n: u64) {
        c.fetch_add(n, Ordering::Relaxed);
    }

    #[inline]
    fn load(c: &CachePadded<AtomicU64>) -> u64 {
        c.load(Ordering::Relaxed)
    }

    /// Takes a snapshot of all current metric values.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            service_startups: Self::load(&self.service_startups),
            service_shutdowns: Self::load(&self.service_shutdowns),

            network_changes: Self::load(&self.network_changes),

            connections_opened: Self::load(&self.connections_opened),
            connections_closed: Self::load(&self.connections_closed),

            dns_requests: Self::load(&self.dns_requests),
            dns_cache_hits: Self::load(&self.dns_cache_hits),
            dns_cache_misses: Self::load(&self.dns_cache_misses),

            proxy_connected: Self::load(&self.proxy_connected),
            proxy_failed: Self::load(&self.proxy_failed),

            task_startups: Self::load(&self.task_startups),
            task_shutdowns: Self::load(&self.task_shutdowns),

            capability_enabled: Self::load(&self.capability_enabled),
            capability_disabled: Self::load(&self.capability_disabled),

            load_initial_proxy: Self::load(&self.load_initial_proxy),
            rotate_proxy: Self::load(&self.rotate_proxy),
            routing_decisions: Self::load(&self.routing_decisions),

            info_events: Self::load(&self.info_events),
            errors: Self::load(&self.errors),

            dns_cache_cleanup: Self::load(&self.dns_cache_cleanup),

            lagged_critical: Self::load(&self.lagged_critical),
            lagged_telemetry: Self::load(&self.lagged_telemetry),
            lagged_lifecycle: Self::load(&self.lagged_lifecycle),
            lagged_diagnostic: Self::load(&self.lagged_diagnostic),
        }
    }

    /// Increments the service startup counter.
    pub fn service_startup(&self) {
        Self::bump(&self.service_startups);
    }

    /// Increments the service shutdown counter.
    pub fn service_shutdown(&self) {
        Self::bump(&self.service_shutdowns);
    }

    /// Increments the network change counter.
    pub fn network_change(&self) {
        Self::bump(&self.network_changes);
    }

    /// Increments the connection opened counter.
    pub fn connection_opened(&self) {
        Self::bump(&self.connections_opened);
    }

    /// Increments the connection closed counter.
    pub fn connection_closed(&self) {
        Self::bump(&self.connections_closed);
    }

    /// Increments the DNS request counter.
    pub fn dns_request(&self) {
        Self::bump(&self.dns_requests);
    }

    /// Increments the DNS cache hit counter.
    pub fn dns_cache_hit(&self) {
        Self::bump(&self.dns_cache_hits);
    }

    /// Increments the DNS cache miss counter.
    pub fn dns_cache_miss(&self) {
        Self::bump(&self.dns_cache_misses);
    }

    /// Increments the proxy connected counter.
    pub fn proxy_connected(&self) {
        Self::bump(&self.proxy_connected);
    }

    /// Increments the proxy failed counter.
    pub fn proxy_failed(&self) {
        Self::bump(&self.proxy_failed);
    }

    /// Increments the task startup counter.
    pub fn task_startup(&self) {
        Self::bump(&self.task_startups);
    }

    /// Increments the task shutdown counter.
    pub fn task_shutdown(&self) {
        Self::bump(&self.task_shutdowns);
    }

    /// Increments the capability enabled counter.
    pub fn capability_enabled(&self) {
        Self::bump(&self.capability_enabled);
    }

    /// Increments the capability disabled counter.
    pub fn capability_disabled(&self) {
        Self::bump(&self.capability_disabled);
    }

    /// Increments the load initial proxy counter.
    pub fn load_initial_proxy(&self) {
        Self::bump(&self.load_initial_proxy);
    }

    /// Increments the proxy rotation counter.
    pub fn rotate_proxy(&self) {
        Self::bump(&self.rotate_proxy);
    }

    /// Increments the routing decision counter.
    pub fn routing_decision(&self) {
        Self::bump(&self.routing_decisions);
    }

    /// Increments the info event counter.
    pub fn info_event(&self) {
        Self::bump(&self.info_events);
    }

    /// Increments the error counter.
    pub fn error(&self) {
        Self::bump(&self.errors);
    }

    /// Adds to the DNS cache cleanup counter.
    ///
    /// # Arguments
    /// * `n` - The number of entries that were cleaned up.
    pub fn dns_cache_cleanup(&self, n: usize) {
        Self::add(&self.dns_cache_cleanup, n as u64);
    }

    /// Adds to the lagged critical events counter.
    ///
    /// # Arguments
    /// * `n` - The number of dropped critical events.
    pub fn lagged_critical(&self, n: u64) {
        Self::add(&self.lagged_critical, n);
    }

    /// Adds to the lagged telemetry events counter.
    ///
    /// # Arguments
    /// * `n` - The number of dropped telemetry events.
    pub fn lagged_telemetry(&self, n: u64) {
        Self::add(&self.lagged_telemetry, n);
    }

    /// Adds to the lagged lifecycle events counter.
    ///
    /// # Arguments
    /// * `n` - The number of dropped lifecycle events.
    pub fn lagged_lifecycle(&self, n: u64) {
        Self::add(&self.lagged_lifecycle, n);
    }

    /// Adds to the lagged diagnostic events counter.
    ///
    /// # Arguments
    /// * `n` - The number of dropped diagnostic events.
    pub fn lagged_diagnostic(&self, n: u64) {
        Self::add(&self.lagged_diagnostic, n);
    }
}
