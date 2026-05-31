use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::metrics::metrics::{Metrics, MetricsSnapshot};
use crate::monitor::bus::Bus;
use crate::monitor::events::{CriticalEvent, DiagnosticEvent, LifecycleEvent, TelemetryEvent};

fn record_critical(metrics: &Metrics, event: &CriticalEvent) {
    match event {
        CriticalEvent::RoutingDecision => metrics.routing_decision(),

        CriticalEvent::RotateProxy => metrics.rotate_proxy(),

        CriticalEvent::BadProxy => metrics.proxy_failed(),

        CriticalEvent::LoadInitialProxy => metrics.load_initial_proxy(),

        CriticalEvent::EnableCapability { .. } => metrics.capability_enabled(),

        CriticalEvent::DisableCapability { .. } => metrics.capability_disabled(),

        CriticalEvent::NetworkChange { .. } => metrics.network_change(),
    }
}

fn record_telemetry(metrics: &Metrics, event: &TelemetryEvent) {
    match event {
        TelemetryEvent::ConnectionOpened { .. } => metrics.connection_opened(),

        TelemetryEvent::ConnectionClosed { .. } => metrics.connection_closed(),

        TelemetryEvent::ProxyConnected { .. } => metrics.proxy_connected(),

        TelemetryEvent::ProxyFailed { .. } => metrics.proxy_failed(),

        TelemetryEvent::DNSRequest { .. } => metrics.dns_request(),

        TelemetryEvent::DNSCacheHit { .. } => metrics.dns_cache_hit(),

        TelemetryEvent::DNSCacheMiss { .. } => metrics.dns_cache_miss(),
    }
}

fn record_lifecycle(metrics: &Metrics, event: &LifecycleEvent) {
    match event {
        LifecycleEvent::ServiceStartup { .. } => metrics.service_startup(),

        LifecycleEvent::ServiceShutdown { .. } => metrics.service_shutdown(),

        LifecycleEvent::TaskStartup { .. } => metrics.task_startup(),

        LifecycleEvent::TaskShutdown { .. } => metrics.task_shutdown(),

        LifecycleEvent::DNSCacheCleanup {
            entries_cleaned, ..
        } => {
            metrics.dns_cache_cleanup(*entries_cleaned);
        }
    }
}

fn record_diagnostic(metrics: &Metrics, event: &DiagnosticEvent) {
    match event {
        DiagnosticEvent::Info { .. } => metrics.info_event(),

        DiagnosticEvent::Error { .. } => metrics.error(),
    }
}

fn print_metrics(current: &MetricsSnapshot, previous: &MetricsSnapshot, elapsed: Duration) {
    let secs = elapsed.as_secs_f64().max(0.001);
    let rate = |now: u64, prev: u64| (now.saturating_sub(prev) as f64) / secs;

    let total_cache = current.dns_cache_hits + current.dns_cache_misses;
    let hit_rate = if total_cache > 0 {
        (current.dns_cache_hits as f64 / total_cache as f64) * 100.0
    } else {
        0.0
    };

    println!();
    println!("========== METRICS ==========");

    println!(
        "services up/down    : {} / {}",
        current.service_startups, current.service_shutdowns
    );
    println!(
        "tasks up/down       : {} / {}",
        current.task_startups, current.task_shutdowns
    );
    println!("network changes     : {}", current.network_changes);

    println!(
        "connections opened  : {} ({:.2}/s)",
        current.connections_opened,
        rate(current.connections_opened, previous.connections_opened)
    );
    println!(
        "connections closed  : {} ({:.2}/s)",
        current.connections_closed,
        rate(current.connections_closed, previous.connections_closed)
    );

    println!(
        "dns requests        : {} ({:.2}/s)",
        current.dns_requests,
        rate(current.dns_requests, previous.dns_requests)
    );
    println!(
        "dns cache hits      : {} ({:.2}/s)",
        current.dns_cache_hits,
        rate(current.dns_cache_hits, previous.dns_cache_hits)
    );
    println!(
        "dns cache misses    : {} ({:.2}/s)",
        current.dns_cache_misses,
        rate(current.dns_cache_misses, previous.dns_cache_misses)
    );
    println!("cache hit rate      : {:.2}%", hit_rate);

    println!(
        "proxy connected     : {} ({:.2}/s)",
        current.proxy_connected,
        rate(current.proxy_connected, previous.proxy_connected)
    );
    println!(
        "proxy failed        : {} ({:.2}/s)",
        current.proxy_failed,
        rate(current.proxy_failed, previous.proxy_failed)
    );

    println!(
        "routing decisions   : {} ({:.2}/s)",
        current.routing_decisions,
        rate(current.routing_decisions, previous.routing_decisions)
    );
    println!(
        "rotate proxy        : {} ({:.2}/s)",
        current.rotate_proxy,
        rate(current.rotate_proxy, previous.rotate_proxy)
    );
    println!(
        "initial proxy loads : {} ({:.2}/s)",
        current.load_initial_proxy,
        rate(current.load_initial_proxy, previous.load_initial_proxy)
    );

    println!(
        "cap enabled/disabled: {} / {}",
        current.capability_enabled, current.capability_disabled
    );

    println!(
        "info / errors       : {} / {}",
        current.info_events, current.errors
    );

    println!("dns cleanup entries : {}", current.dns_cache_cleanup);

    println!("lagged critical     : {}", current.lagged_critical);
    println!("lagged telemetry    : {}", current.lagged_telemetry);
    println!("lagged lifecycle    : {}", current.lagged_lifecycle);
    println!("lagged diagnostic   : {}", current.lagged_diagnostic);

    println!("=============================");
    println!();
}

/// Aggregates events from the bus into metrics and periodically prints statistics.
pub async fn start_metrics(
    metrics: Arc<Metrics>,
    bus: Arc<Bus>,
    cancel: CancellationToken,
) -> Result<()> {
    let mut critical_rx = bus.subscribe_critical();

    let mut lifecycle_rx = bus.subscribe_lifecycle();

    let mut diagnostic_rx = bus.subscribe_diagnostic();

    let telemetry_rx = bus.telemetry_receiver();

    let mut ticker = interval(Duration::from_secs(10));

    let mut previous_snapshot = metrics.snapshot();

    let mut previous_tick = Instant::now();

    let mut critical_open = true;
    let mut telemetry_open = true;
    let mut lifecycle_open = true;
    let mut diagnostic_open = true;

    loop {
        tokio::select! {
            biased;

            _ = cancel.cancelled() => {
                break;
            }

            _ = ticker.tick() => {
                let now = Instant::now();

                let current =
                    metrics.snapshot();

                let elapsed =
                    now.duration_since(
                        previous_tick
                    );

                print_metrics(
                    &current,
                    &previous_snapshot,
                    elapsed,
                );

                previous_snapshot =
                    current;

                previous_tick = now;
            }

            result = critical_rx.recv(),
            if critical_open => {
                match result {
                    Ok(event) => {
                        record_critical(
                            &metrics,
                            &event,
                        );
                    }

                    Err(
                        broadcast::error::RecvError::Lagged(
                            skipped,
                        )
                    ) => {
                        metrics
                            .lagged_critical(
                                skipped
                            );
                    }

                    Err(
                        broadcast::error::RecvError::Closed
                    ) => {
                        critical_open = false;
                    }
                }
            }

            result = telemetry_rx.recv(),
            if telemetry_open => {
                match result {
                    Ok(event) => {
                        record_telemetry(
                            &metrics,
                            &event,
                        );
                    }

                    Err(_) => {
                        telemetry_open = false;
                    }
                }
            }

            result = lifecycle_rx.recv(),
            if lifecycle_open => {
                match result {
                    Ok(event) => {
                        record_lifecycle(
                            &metrics,
                            &event,
                        );
                    }

                    Err(
                        broadcast::error::RecvError::Lagged(
                            skipped,
                        )
                    ) => {
                        metrics
                            .lagged_lifecycle(
                                skipped
                            );
                    }

                    Err(
                        broadcast::error::RecvError::Closed
                    ) => {
                        lifecycle_open = false;
                    }
                }
            }

            result = diagnostic_rx.recv(),
            if diagnostic_open => {
                match result {
                    Ok(event) => {
                        record_diagnostic(
                            &metrics,
                            &event,
                        );
                    }

                    Err(
                        broadcast::error::RecvError::Lagged(
                            skipped,
                        )
                    ) => {
                        metrics
                            .lagged_diagnostic(
                                skipped
                            );
                    }

                    Err(
                        broadcast::error::RecvError::Closed
                    ) => {
                        diagnostic_open = false;
                    }
                }
            }
        }

        if !critical_open && !telemetry_open && !lifecycle_open && !diagnostic_open {
            break;
        }
    }

    Ok(())
}
