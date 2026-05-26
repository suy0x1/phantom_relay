use anyhow::Result;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::monitor::{
    bus::Bus,
    events::{CriticalEvent, DiagnosticEvent, LifecycleEvent},
};

fn log_critical(event: CriticalEvent) {
    match event {
        CriticalEvent::NetworkChange { change, timestamp } => {
            println!("[{:?}] [network] {}", timestamp, change);
        }

        CriticalEvent::RoutingDecision => {
            println!("[routing] decision made");
        }

        CriticalEvent::EnableCapability { cap, timestamp } => {
            println!("[{:?}] [capability] {:?} enabled", timestamp, cap);
        }

        CriticalEvent::DisableCapability { cap, timestamp } => {
            println!("[{:?}] [capability] {:?} disabled", timestamp, cap);
        }

        CriticalEvent::LoadInitialProxy => {
            println!("[init] loading first proxy");
        }

        CriticalEvent::RotateProxy => {
            println!("[rotate] rotating proxy now");
        }
    }
}

fn log_lifecycle(event: LifecycleEvent) {
    match event {
        LifecycleEvent::ServiceStartup {
            service_name,
            port,
            timestamp,
        } => {
            println!(
                "[{:?}] [service] {} started on {}",
                timestamp, service_name, port,
            );
        }

        LifecycleEvent::ServiceShutdown {
            service_name,
            port,
            timestamp,
        } => {
            println!(
                "[{:?}] [service] {} stopped on {}",
                timestamp, service_name, port,
            );
        }

        LifecycleEvent::TaskStartup {
            task_name,
            timestamp,
        } => {
            println!("[{:?}] [task] {} started", timestamp, task_name,);
        }

        LifecycleEvent::TaskShutdown {
            task_name,
            timestamp,
        } => {
            println!("[{:?}] [task] {} stopped", timestamp, task_name,);
        }

        LifecycleEvent::DNSCacheCleanup {
            entries_cleaned,
            timestamp,
        } => {
            println!(
                "[{:?}] [dns-cache] cleaned {} entries",
                timestamp, entries_cleaned,
            );
        }
    }
}

fn log_diagnostic(event: DiagnosticEvent) {
    match event {
        DiagnosticEvent::Info { content, timestamp } => {
            println!("[{:?}] [info] {}", timestamp, content,);
        }

        DiagnosticEvent::Error { err, timestamp } => {
            eprintln!("[{:?}] [error] {}", timestamp, err,);
        }
    }
}

pub async fn start_logger(bus: Arc<Bus>, cancel: CancellationToken) -> Result<()> {
    let mut critical_rx = bus.subscribe_critical();

    let lifecycle_rx = bus.lifecycle_receiver();

    let diagnostic_rx = bus.diagnostic_receiver();

    let mut critical_open = true;
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
                        log_critical(event);
                    }

                    Err(
                        broadcast::error::RecvError::Lagged(
                            skipped,
                        )
                    ) => {
                        eprintln!(
                            "[logger] critical lagged by {} events",
                            skipped,
                        );
                    }

                    Err(
                        broadcast::error::RecvError::Closed
                    ) => {
                        critical_open = false;
                    }
                }
            }

            result = lifecycle_rx.recv(),
            if lifecycle_open => {
                match result {
                    Ok(event) => {
                        log_lifecycle(event);
                    }

                    Err(_) => {
                        lifecycle_open = false;
                    }
                }
            }

            result = diagnostic_rx.recv(),
            if diagnostic_open => {
                match result {
                    Ok(event) => {
                        log_diagnostic(event);
                    }

                    Err(_) => {
                        diagnostic_open = false;
                    }
                }
            }
        }

        if !critical_open && !lifecycle_open && !diagnostic_open {
            break;
        }
    }

    Ok(())
}
