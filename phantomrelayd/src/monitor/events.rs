use std::net::IpAddr;
use std::time::SystemTime;

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

    /// A network capability has been enabled.
    EnableCapability {
        /// The network capability that was enabled.
        cap: NetworkCapability,
        /// When the capability was enabled.
        timestamp: SystemTime,
    },

    /// A network capability has been disabled.
    DisableCapability {
        /// The network capability that was disabled.
        cap: NetworkCapability,
        /// When the capability was disabled.
        timestamp: SystemTime,
    },

    /// A network change has been detected.
    NetworkChange {
        /// Description of the network change.
        change: String,
        /// When the change was detected.
        timestamp: SystemTime,
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

        /// When the connection was opened.
        timestamp: SystemTime,
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

        /// When the connection was closed.
        timestamp: SystemTime,
    },

    /// Successfully connected to a proxy server.
    ProxyConnected {
        /// The proxy IP address.
        host: IpAddr,
        /// The proxy port.
        port: u16,

        /// When the connection succeeded.
        timestamp: SystemTime,
    },

    /// Failed to connect to a proxy server.
    ProxyFailed {
        /// The proxy IP address.
        host: IpAddr,
        /// The proxy port.
        port: u16,

        /// When the connection failed.
        timestamp: SystemTime,
    },

    /// A DNS query was made.
    DNSRequest {
        /// The domain being queried.
        domain: String,
        /// The resolver IP address that answered the query.
        resolver: IpAddr,

        /// When the query was made.
        timestamp: SystemTime,
    },

    /// A DNS query result was found in the cache.
    DNSCacheHit {
        /// The domain that was cached.
        domain: String,
        /// When the cache hit occurred.
        timestamp: SystemTime,
    },

    /// A DNS query result was not found in the cache.
    DNSCacheMiss {
        /// The domain that was not cached.
        domain: String,
        /// When the cache miss occurred.
        timestamp: SystemTime,
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

        /// When the service started.
        timestamp: SystemTime,
    },

    /// A service has shut down.
    ServiceShutdown {
        /// The name of the service that shut down.
        service_name: String,
        /// The port the service was listening on.
        port: u16,

        /// When the service shut down.
        timestamp: SystemTime,
    },

    /// A background task has started.
    TaskStartup {
        /// The name of the task that started.
        task_name: String,

        /// When the task started.
        timestamp: SystemTime,
    },

    /// A background task has shut down.
    TaskShutdown {
        /// The name of the task that shut down.
        task_name: String,

        /// When the task shut down.
        timestamp: SystemTime,
    },

    /// The DNS cache has been cleaned of expired entries.
    DNSCacheCleanup {
        /// The number of entries removed from the cache.
        entries_cleaned: usize,

        /// When the cleanup occurred.
        timestamp: SystemTime,
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

        /// When the message was generated.
        timestamp: SystemTime,
    },

    /// An error message.
    Error {
        /// The error description.
        err: String,

        /// When the error occurred.
        timestamp: SystemTime,
    },
}
