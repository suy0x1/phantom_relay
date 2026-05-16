use anyhow::Result;
use dashmap::DashMap;

use crate::dns::cache::CacheEntry;
use crate::dns::listener::start_dns_listener;
use crate::redirect::listener::start_listener;
use crate::relay::manager::ConnectionManager;
use crate::system::nftables::network_setup;
use std::sync::Arc;

pub async fn startup(proxy_port: u16, dns_port: u16) -> Result<()> {
    let conn_map = Arc::new(ConnectionManager::new());
    let dns_cache: Arc<DashMap<Vec<u8>, CacheEntry>> = Arc::new(DashMap::new());
    
    network_setup(proxy_port, dns_port)?;
    let dns_cache = dns_cache.clone();
    tokio::spawn(async move {
        if let Err(e) = start_dns_listener(dns_cache).await {
            println!("listener error {}", e);
        }
    });

    let conn_map = conn_map.clone();
    tokio::spawn(async move {
        if let Err(e) = start_listener(conn_map.clone()).await {
            println!("listener error {}", e);
        }
    });

    Ok(())
}
