use crate::monitor::bus::Bus;
use crate::monitor::events::DiagnosticEvent;
use std::sync::Arc;

/// Extension trait for converting errors to bus diagnostic events.
///
/// Allows any `Result` type to emit errors to the event bus and convert them to anyhow errors.
pub trait BusErrorExt<T> {
    /// Emits an error to the bus as a diagnostic event and converts it to an anyhow error.
    ///
    /// If the result is `Ok`, the value is returned unchanged. If the result is `Err`,
    /// the error is formatted and sent as a diagnostic event to the bus before being
    /// converted to an anyhow error.
    ///
    /// # Arguments
    /// * `bus` - The event bus to emit the diagnostic event to.
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
                });
                Err(anyhow::anyhow!(err_msg))
            }
        }
    }
}
