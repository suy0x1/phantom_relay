use std::net::IpAddr;

use crate::subsystems::network::capablities::NetworkCapability;

/// Critical events that require immediate attention or action.
///
/// These events represent significant state changes in the relay system,
/// such as proxy rotations, network capability changes, or routing decisions.
#[derive(Debug, Clone)]
pub enum CriticalEvent {
    /// A routing decision was made for a connection.
    RoutingDecision,

    /// The active proxy is being rotated to a different one.
    RotateProxy,

    /// The initial proxy connection has been loaded.
    LoadInitialProxy,

    /// The initial proxy connection has been loaded.
    BadProxy,

    /// A network capability has been enabled.
    EnableCapability {
        /// The network capability that was enabled.
        cap: NetworkCapability,
    },

    /// A network capability has been disabled.
    DisableCapability {
        /// The network capability that was disabled.
        cap: NetworkCapability,
    },

    /// A network change has been detected.
    NetworkChange {
        /// Description of the network change.
        change: String,
    },
}

/// Telemetry events that track connection and DNS activity.
///
/// These events provide detailed metrics about individual connections and DNS queries.
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    /// A connection to a target host has been opened.
    ConnectionOpened {
        /// The destination host IP address.
        host: IpAddr,
        /// The destination port.
        port: u16,

        /// The proxy IP address being used.
        proxy: IpAddr,
        /// The proxy port.
        proxy_port: u16,
    },

    /// A connection to a target host has been closed.
    ConnectionClosed {
        /// The destination host IP address.
        host: IpAddr,
        /// The destination port.
        port: u16,

        /// The proxy IP address that was used.
        proxy: IpAddr,
        /// The proxy port.
        proxy_port: u16,
    },

    /// Successfully connected to a proxy server.
    ProxyConnected {
        /// The proxy IP address.
        host: IpAddr,
        /// The proxy port.
        port: u16,
    },

    /// Failed to connect to a proxy server.
    ProxyFailed {
        /// The proxy IP address.
        host: IpAddr,
        /// The proxy port.
        port: u16,
    },

    /// A DNS query was made.
    DNSRequest {
        /// The domain being queried.
        domain: String,
        /// The resolver IP address that answered the query.
        resolver: IpAddr,
    },

    /// A DNS query result was found in the cache.
    DNSCacheHit {
        /// The domain that was cached.
        domain: String,
    },

    /// A DNS query result was not found in the cache.
    DNSCacheMiss {
        /// The domain that was not cached.
        domain: String,
    },
}

/// Lifecycle events that track service and task state transitions.
///
/// These events indicate when services and background tasks start and stop.
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// A service has started.
    ServiceStartup {
        /// The name of the service that started.
        service_name: String,
        /// The port the service is listening on.
        port: u16,
    },

    /// A service has shut down.
    ServiceShutdown {
        /// The name of the service that shut down.
        service_name: String,
        /// The port the service was listening on.
        port: u16,
    },

    /// A background task has started.
    TaskStartup {
        /// The name of the task that started.
        task_name: String,
    },

    /// A background task has shut down.
    TaskShutdown {
        /// The name of the task that shut down.
        task_name: String,
    },

    /// The DNS cache has been cleaned of expired entries.
    DNSCacheCleanup {
        /// The number of entries removed from the cache.
        entries_cleaned: usize,
    },
}

/// Diagnostic events for logging and debugging.
///
/// These events include informational messages and error reports.
#[derive(Debug, Clone)]
pub enum DiagnosticEvent {
    /// An informational message.
    Info {
        /// The diagnostic message content.
        content: String,
    },

    /// An error message.
    Error {
        /// The error description.
        err: String,
    },
}
