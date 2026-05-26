use std::net::IpAddr;
use std::time::SystemTime;

use crate::subsystems::network::capablities::NetworkCapability;

#[derive(Debug, Clone)]
pub enum CriticalEvent {
    RoutingDecision,

    RotateProxy,

    LoadInitialProxy,

    EnableCapability {
        cap: NetworkCapability,
        timestamp: SystemTime,
    },

    DisableCapability {
        cap: NetworkCapability,
        timestamp: SystemTime,
    },

    NetworkChange {
        change: String,
        timestamp: SystemTime,
    },
}

#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    ConnectionOpened {
        host: IpAddr,
        port: u16,

        proxy: IpAddr,
        proxy_port: u16,

        timestamp: SystemTime,
    },

    ConnectionClosed {
        host: IpAddr,
        port: u16,

        proxy: IpAddr,
        proxy_port: u16,

        timestamp: SystemTime,
    },

    ProxyConnected {
        host: IpAddr,
        port: u16,

        timestamp: SystemTime,
    },

    ProxyFailed {
        host: IpAddr,
        port: u16,

        timestamp: SystemTime,
    },

    DNSRequest {
        domain: String,
        resolver: IpAddr,

        timestamp: SystemTime,
    },

    DNSCacheHit {
        domain: String,
        timestamp: SystemTime,
    },

    DNSCacheMiss {
        domain: String,
        timestamp: SystemTime,
    },
}

#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    ServiceStartup {
        service_name: String,
        port: u16,

        timestamp: SystemTime,
    },

    ServiceShutdown {
        service_name: String,
        port: u16,

        timestamp: SystemTime,
    },

    TaskStartup {
        task_name: String,

        timestamp: SystemTime,
    },

    TaskShutdown {
        task_name: String,

        timestamp: SystemTime,
    },

    DNSCacheCleanup {
        entries_cleaned: usize,

        timestamp: SystemTime,
    },
}

#[derive(Debug, Clone)]
pub enum DiagnosticEvent {
    Info {
        content: String,

        timestamp: SystemTime,
    },

    Error {
        err: String,

        timestamp: SystemTime,
    },
}
