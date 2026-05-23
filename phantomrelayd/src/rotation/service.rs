use crate::rotation::manager::HealthyProxy;

use super::collector::get_proxy;
use std::sync::{Arc, atomic::AtomicU32};
use super::health::get_healthy_proxies;
use anyhow::Result;
use dashmap::DashMap;
use reqwest::Client;
use tokio::task::JoinSet;

fn divide_round_robin(items: Vec<String>, n: usize) -> Vec<Vec<String>> {
    if n == 0 || items.is_empty() {
        return Vec::new();
    }

    let mut pools = vec![Vec::new(); n];

    for (index, item) in items.into_iter().enumerate() {
        pools[index % n].push(item);
    }

    pools
}

pub async fn async_health_check(progress: Arc<AtomicU32>) -> Result<()> {
    let proxies = get_proxy().await?;
    let work = divide_round_robin(proxies, 400);
    let healthy_proxies: Arc<DashMap<HealthyProxy, Client>> = Arc::new(DashMap::new());
    let mut set = JoinSet::new();
    for i in work {
        let hp = healthy_proxies.clone();
        let prog = progress.clone();
        set.spawn(async move {
            get_healthy_proxies(hp,5000,i,prog).await
        });
    }

    while let Some(res) = set.join_next().await{
        if let Err(e) = res {
            println!("{e}")
        }
    }
    Ok(())
}

