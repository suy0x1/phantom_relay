use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_utils::CachePadded;

#[derive(Debug)]
pub struct Metrics {
    pub service_startups: CachePadded<AtomicU64>,
    pub service_shutdowns: CachePadded<AtomicU64>,

    pub network_changes: CachePadded<AtomicU64>,

    pub connections_opened: CachePadded<AtomicU64>,
    pub connections_closed: CachePadded<AtomicU64>,

    pub dns_requests: CachePadded<AtomicU64>,
    pub dns_cache_hits: CachePadded<AtomicU64>,
    pub dns_cache_misses: CachePadded<AtomicU64>,

    pub proxy_connected: CachePadded<AtomicU64>,
    pub proxy_failed: CachePadded<AtomicU64>,

    pub task_startups: CachePadded<AtomicU64>,
    pub task_shutdowns: CachePadded<AtomicU64>,

    pub capability_enabled: CachePadded<AtomicU64>,
    pub capability_disabled: CachePadded<AtomicU64>,

    pub load_initial_proxy: CachePadded<AtomicU64>,
    pub rotate_proxy: CachePadded<AtomicU64>,
    pub routing_decisions: CachePadded<AtomicU64>,

    pub info_events: CachePadded<AtomicU64>,
    pub errors: CachePadded<AtomicU64>,

    pub dns_cache_cleanup: CachePadded<AtomicU64>,

    pub lagged_critical: CachePadded<AtomicU64>,
    pub lagged_telemetry: CachePadded<AtomicU64>,
    pub lagged_lifecycle: CachePadded<AtomicU64>,
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

    pub fn service_startup(&self) {
        Self::bump(&self.service_startups);
    }

    pub fn service_shutdown(&self) {
        Self::bump(&self.service_shutdowns);
    }

    pub fn network_change(&self) {
        Self::bump(&self.network_changes);
    }

    pub fn connection_opened(&self) {
        Self::bump(&self.connections_opened);
    }

    pub fn connection_closed(&self) {
        Self::bump(&self.connections_closed);
    }

    pub fn dns_request(&self) {
        Self::bump(&self.dns_requests);
    }

    pub fn dns_cache_hit(&self) {
        Self::bump(&self.dns_cache_hits);
    }

    pub fn dns_cache_miss(&self) {
        Self::bump(&self.dns_cache_misses);
    }

    pub fn proxy_connected(&self) {
        Self::bump(&self.proxy_connected);
    }

    pub fn proxy_failed(&self) {
        Self::bump(&self.proxy_failed);
    }

    pub fn task_startup(&self) {
        Self::bump(&self.task_startups);
    }

    pub fn task_shutdown(&self) {
        Self::bump(&self.task_shutdowns);
    }

    pub fn capability_enabled(&self) {
        Self::bump(&self.capability_enabled);
    }

    pub fn capability_disabled(&self) {
        Self::bump(&self.capability_disabled);
    }

    pub fn load_initial_proxy(&self) {
        Self::bump(&self.load_initial_proxy);
    }

    pub fn rotate_proxy(&self) {
        Self::bump(&self.rotate_proxy);
    }

    pub fn routing_decision(&self) {
        Self::bump(&self.routing_decisions);
    }

    pub fn info_event(&self) {
        Self::bump(&self.info_events);
    }

    pub fn error(&self) {
        Self::bump(&self.errors);
    }

    pub fn dns_cache_cleanup(&self, n: usize) {
        Self::add(&self.dns_cache_cleanup, n as u64);
    }

    pub fn lagged_critical(&self, n: u64) {
        Self::add(&self.lagged_critical, n);
    }

    pub fn lagged_telemetry(&self, n: u64) {
        Self::add(&self.lagged_telemetry, n);
    }

    pub fn lagged_lifecycle(&self, n: u64) {
        Self::add(&self.lagged_lifecycle, n);
    }

    pub fn lagged_diagnostic(&self, n: u64) {
        Self::add(&self.lagged_diagnostic, n);
    }
}
