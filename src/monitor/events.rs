use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum Event {
    ServiceStartup {
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
}
