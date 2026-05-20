use std::{net::IpAddr, time::Instant};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ConnectionKey {
    pub dst_ip: IpAddr,
    pub dst_port: u16,
}

#[derive(Debug)]
pub struct ProxyConnection {
    pub started: Instant,
    pub proxy: String,
    
}