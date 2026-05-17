use anyhow::Result;
use dashmap::DashMap;

use crate::dns::cache::CacheEntry;
use crate::dns::listener::start_dns_listener;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event::Error;
use crate::monitor::logger::start_logger;
use crate::redirect::listener::start_listener;
use crate::relay::manager::ConnectionManager;
use crate::system::nftables::network_setup;
use chrono::Local;
use std::sync::Arc;

pub async fn startup(proxy_port: u16, dns_port: u16, bus: Arc<Bus>) -> Result<()> {
    let conn_map = Arc::new(ConnectionManager::new());
    let dns_cache: Arc<DashMap<Vec<u8>, CacheEntry>> = Arc::new(DashMap::new());
    let bus_clone = bus.clone();
    tokio::spawn(async move {
        if let Err(e) = start_logger(bus_clone.clone()).await {
            let _ = bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()});
        }
    });

    network_setup(proxy_port, dns_port, bus.clone())?;

    let bus_clone = bus.clone();
    let dns_cache = dns_cache.clone();
    tokio::spawn(async move {
        if let Err(e) = start_dns_listener(dns_cache, bus_clone.clone()).await {
            let _ = bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()});
        }
    });

    let conn_map = conn_map.clone();
    let bus_clone = bus.clone();
    tokio::spawn(async move {
        if let Err(e) = start_listener(conn_map.clone(), bus.clone()).await {
            let _ = bus_clone.emit(Error{err:format!("{}", e), timestamp: Local::now().format("%H:%M:%S").to_string().to_string()});
        }
    });

    Ok(())
}
