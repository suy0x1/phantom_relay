use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::metrics::metrics::Metrics;
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
                        metrics.lagged_critical(
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
                        metrics.lagged_lifecycle(
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
                        metrics.lagged_diagnostic(
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

        if !critical_open
            && !telemetry_open
            && !lifecycle_open
            && !diagnostic_open
        {
            break;
        }
    }

    Ok(())
}