use crate::{
    config::collector::CollectorConfig,
    monitor::bus::Bus,
    monitor::events::{LifecycleEvent, DiagnosticEvent},
    collector::manager::HealthyProxy,
};

use super::collector::get_proxy;
use super::health::get_healthy_proxies;
use anyhow::Result;
use std::time::SystemTime;
use dashmap::DashMap;
use reqwest::Client;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::interval};
use tokio_util::sync::CancellationToken;

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

pub async fn collect_healthy_proxy(
    config: Arc<Mutex<CollectorConfig>>,
    bus: Arc<Bus>,
    healthy_proxies: Arc<DashMap<HealthyProxy, Client>>,
    cancel: CancellationToken,
) -> Result<()> {
    bus.emit_lifecycle(LifecycleEvent::TaskStartup {
        task_name: "proxy_collector".to_string(),
        timestamp: SystemTime::now(),
    }).await;

    let mut ticker = interval(Duration::from_mins(45));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                bus.emit_lifecycle(LifecycleEvent::TaskShutdown {
                    task_name: "proxy_collector".to_string(),
                    timestamp: SystemTime::now(),
                }).await;

                break;
            }

            _ = ticker.tick() => {
                let proxies = match get_proxy(&bus, cancel.clone()).await {
                    Ok(p) => p,

                    Err(e) => {
                        bus.emit_diagnostic(DiagnosticEvent::Error {
                            err: format!("{:#?}", e),
                            timestamp: SystemTime::now(),
                        });

                        continue;
                    }
                };

                let (workers, latency) = {
                    let cfg = config.lock().await;
                    (cfg.total_workers, cfg.latency)
                };

                let work = divide_round_robin(proxies, workers);

                for i in work {
                    let hp = healthy_proxies.clone();
                    let cancel_clone = cancel.clone();

                    tokio::spawn(async move {
                        let _ = get_healthy_proxies(
                            hp,
                            latency,
                            i,
                            cancel_clone
                        ).await;
                    });
                }
            }
        }
    }

    Ok(())
}
