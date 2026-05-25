use std::net::IpAddr;

use crate::subsystems::network::capablities::NetworkCapability;

#[derive(Debug, Clone)]
pub enum Event {
    ServiceStartup {
        service_name: String,
        port: u16,
        timestamp: String,
    },

    ServiceShutdown {
        service_name: String,
        port: u16,
        timestamp: String,
    },
    NetworkChange {
        change: String,
        timestamp: String,
    },

    ConnectionOpened {
        host: IpAddr,
        port: u16,
        proxy: IpAddr,
        proxy_port: u16,
        timestamp: String,
    },
    ConnectionClosed {
        host: IpAddr,
        port: u16,
        proxy: IpAddr,
        proxy_port: u16,
        timestamp: String,
    },

    DNSRequest {
        domain: String,
        resolver: IpAddr,
        timestamp: String,
    },
    DNSCacheHit {
        domain: String,
        timestamp: String,
    },
    DNSCacheMiss {
        domain: String,
        timestamp: String,
    },

    ProxyConnected {
        host: IpAddr,
        port: u16,
        timestamp: String
    },
    ProxyFailed {
        host: IpAddr,
        port: u16,
        timestamp: String
    },

    RoutingDecision,

    Error {
        err: String,
        timestamp: String
    },

    DNSCacheCleanup {
        entries_cleaned: usize,
        timestamp: String,
    },

    TaskStartup {
        task_name: String,
        timestamp: String,
    },

    TaskShutdown {
        task_name: String,
        timestamp: String,
    },

    Info {
        content: String,
        timestamp: String,
    },

    EnableCapability {
        cap: NetworkCapability,
        timestamp: String,
    },

    DisableCapability {
        cap: NetworkCapability,
        timestamp: String,
    },

    LoadInitialProxy {
        timestamp: String,
    },

    RotateProxy {
        timestamp: String,
    },

}
