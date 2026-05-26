use crate::monitor::bus::Bus;
use crate::monitor::events::DiagnosticEvent;
use std::sync::Arc;
use std::time::SystemTime;

pub trait BusErrorExt<T> {
    fn emit_to_bus(self, bus: &Arc<Bus>) -> anyhow::Result<T>;
}

impl<T, E> BusErrorExt<T> for Result<T, E>
where
    E: std::fmt::Display + Send + Sync + 'static,
{
    fn emit_to_bus(self, bus: &Arc<Bus>) -> anyhow::Result<T> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => {
                let err_msg = format!("{}", e);
                let _ = bus.emit_diagnostic(DiagnosticEvent::Error {
                    err: err_msg.clone(),
                    timestamp: SystemTime::now(),
                });
                Err(anyhow::anyhow!(err_msg))
            }
        }
    }
}
