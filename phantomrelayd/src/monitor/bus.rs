use anyhow::Result;
use kanal::{AsyncReceiver, AsyncSender};
use tokio::sync::broadcast;

use crate::monitor::events::{CriticalEvent, DiagnosticEvent, LifecycleEvent, TelemetryEvent};

#[derive(Clone)]
pub struct Bus {
    critical_tx: broadcast::Sender<CriticalEvent>,
    lifecycle_tx: broadcast::Sender<LifecycleEvent>,
    diagnostic_tx: broadcast::Sender<DiagnosticEvent>,

    telemetry_tx: AsyncSender<TelemetryEvent>,
    telemetry_rx: AsyncReceiver<TelemetryEvent>,
}

impl Bus {
    #[must_use]
    pub fn new(
        critical_capacity: usize,
        telemetry_capacity: usize,
        lifecycle_capacity: usize,
        diagnostic_capacity: usize,
    ) -> Self {
        let (critical_tx, _) = broadcast::channel(critical_capacity);

        let (lifecycle_tx, _) = broadcast::channel(lifecycle_capacity);

        let (diagnostic_tx, _) = broadcast::channel(diagnostic_capacity);

        let (telemetry_tx, telemetry_rx) = kanal::bounded_async(telemetry_capacity);

        Self {
            critical_tx,
            lifecycle_tx,
            diagnostic_tx,

            telemetry_tx,
            telemetry_rx,
        }
    }

    #[must_use]
    pub fn subscribe_critical(&self) -> broadcast::Receiver<CriticalEvent> {
        self.critical_tx.subscribe()
    }

    #[must_use]
    pub fn subscribe_lifecycle(&self) -> broadcast::Receiver<LifecycleEvent> {
        self.lifecycle_tx.subscribe()
    }

    #[must_use]
    pub fn subscribe_diagnostic(&self) -> broadcast::Receiver<DiagnosticEvent> {
        self.diagnostic_tx.subscribe()
    }

    #[must_use]
    pub fn telemetry_receiver(&self) -> AsyncReceiver<TelemetryEvent> {
        self.telemetry_rx.clone()
    }

    pub fn emit_critical(
        &self,
        event: CriticalEvent,
    ) -> Result<usize, broadcast::error::SendError<CriticalEvent>> {
        self.critical_tx.send(event)
    }

    pub fn emit_lifecycle(
        &self,
        event: LifecycleEvent,
    ) -> Result<usize, broadcast::error::SendError<LifecycleEvent>> {
        self.lifecycle_tx.send(event)
    }

    pub fn emit_diagnostic(
        &self,
        event: DiagnosticEvent,
    ) -> Result<usize, broadcast::error::SendError<DiagnosticEvent>> {
        self.diagnostic_tx.send(event)
    }

    pub async fn emit_telemetry(&self, event: TelemetryEvent) {
        let _ = self.telemetry_tx.send(event).await;
    }
}
