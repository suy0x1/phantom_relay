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
        CriticalEvent::NetworkChange { change } => {
            println!("[network] {}", change);
        }

        CriticalEvent::RoutingDecision => {
            println!("[routing] decision made");
        }

        CriticalEvent::EnableCapability { cap } => {
            println!("[capability] {:?} enabled", cap);
        }

        CriticalEvent::DisableCapability { cap } => {
            println!("[capability] {:?} disabled", cap);
        }

        CriticalEvent::LoadInitialProxy => {
            println!("[init] loading first proxy");
        }

        CriticalEvent::RotateProxy => {
            println!("[rotate] requesting to rotate proxy");
        }

        CriticalEvent::BadProxy => {
            println!("[bad proxy] requesting to rotate bad proxy")
        }
    }
}

fn log_lifecycle(event: LifecycleEvent) {
    match event {
        LifecycleEvent::ServiceStartup { service_name, port } => {
            println!("[service] {} started on {}", service_name, port,);
        }

        LifecycleEvent::ServiceShutdown { service_name, port } => {
            println!("[service] {} stopped on {}", service_name, port,);
        }

        LifecycleEvent::TaskStartup { task_name } => {
            println!("[task] {} started", task_name,);
        }

        LifecycleEvent::TaskShutdown { task_name } => {
            println!("[task] {} stopped", task_name,);
        }

        LifecycleEvent::DNSCacheCleanup { entries_cleaned } => {
            println!("[dns-cache] cleaned {} entries", entries_cleaned,);
        }
    }
}

fn log_diagnostic(event: DiagnosticEvent) {
    match event {
        DiagnosticEvent::Info { content } => {
            println!("[info] {}", content,);
        }

        DiagnosticEvent::Error { err } => {
            eprintln!("[error] {}", err,);
        }
    }
}

/// Subscribes to bus events and logs them to stdout. Respects cancellation token.
pub async fn start_logger(bus: Arc<Bus>, cancel: CancellationToken) -> Result<()> {
    let mut critical_rx = bus.subscribe_critical();

    let mut lifecycle_rx = bus.subscribe_lifecycle();

    let mut diagnostic_rx = bus.subscribe_diagnostic();

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
