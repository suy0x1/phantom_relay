use dashmap::DashMap;

use crate::routing::connection::{ConnectionKey, ProxyConnection};

#[derive(Debug)]
pub struct ConnectionManager {
    pub connections: DashMap<ConnectionKey, ProxyConnection>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }
}
