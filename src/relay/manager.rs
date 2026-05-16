use dashmap::DashMap;

use crate::relay::connection::{ConnectionKey, ProxyConnection};

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