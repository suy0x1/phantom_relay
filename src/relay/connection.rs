use std::{net::Ipv4Addr, time::Instant};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ConnectionKey {
    pub dst_ip: Ipv4Addr,
    pub dst_port: u16,
}

#[derive(Debug)]
pub struct ProxyConnection {
    pub started: Instant,
    pub proxy: String,
    
}