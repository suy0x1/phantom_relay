use crate::metrics::metrics::Metrics;
use crate::monitor::bus::Bus;
use crate::monitor::events::Event;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, interval};
use tokio_util::sync::CancellationToken;

async fn record_metrics(metrics: &mut Metrics, event: &Event) {
    match event {
        Event::DNSRequest { .. } => {
            metrics.dns_requests += 1;
        }

        Event::DNSCacheHit { .. } => {
            metrics.dns_cache_hits += 1;
        }

        Event::DNSCacheMiss { .. } => {
            metrics.dns_cache_misses += 1;
        }

        Event::ConnectionOpened { .. } => {
            metrics.connections_opened += 1;
        }

        Event::ConnectionClosed { .. } => {
            metrics.connections_closed += 1;
        }

        Event::ProxyConnected { .. } => {
            metrics.proxy_connected += 1;
        }

        Event::ProxyFailed { .. } => {
            metrics.proxy_failed += 1;
        }

        Event::Error { .. } => {
            metrics.errors += 1;
        }

        _ => {}
    }
}

async fn print_metrics(metrics: &Metrics) {
    println!();
    println!("========== METRICS ==========");

    println!("dns requests        : {}", metrics.dns_requests);
    println!("dns cache hits      : {}", metrics.dns_cache_hits);
    println!("dns cache misses    : {}", metrics.dns_cache_misses);

    let total_cache = metrics.dns_cache_hits + metrics.dns_cache_misses;

    if total_cache > 0 {
        let hit_rate = (metrics.dns_cache_hits as f64 / total_cache as f64) * 100.0;

        println!("cache hit rate      : {:.2}%", hit_rate);
    }

    println!("connections opened  : {}", metrics.connections_opened);
    println!("connections closed  : {}", metrics.connections_closed);

    println!("proxy connected     : {}", metrics.proxy_connected);
    println!("proxy failed        : {}", metrics.proxy_failed);

    println!("errors              : {}", metrics.errors);

    println!("=============================");
    println!();
}

pub async fn start_metrics(bus: Arc<Bus>, cancel: CancellationToken) -> Result<()> {
    let mut rx = bus.subscribe();

    let mut metrics = Metrics::default();

    let mut ticker = interval(Duration::from_secs(10));

    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                break;
            }

            _ = ticker.tick() => {
                print_metrics(&metrics).await;
            }

            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        record_metrics(
                            &mut metrics,
                            &event,
                        )
                        .await;
                    }

                    Err(
                        broadcast::error::RecvError::Lagged(n)
                    ) => {
                        eprintln!(
                            "metrics lagged {} events",
                            n
                        );
                    }

                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
