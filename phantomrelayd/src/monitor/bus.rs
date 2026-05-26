use anyhow::Result;
use kanal::{AsyncReceiver, AsyncSender};
use tokio::sync::broadcast;

use crate::monitor::events::{
    CriticalEvent,
    DiagnosticEvent,
    LifecycleEvent,
    TelemetryEvent,
};

#[derive(Clone)]
pub struct Bus {
    pub critical_tx: broadcast::Sender<CriticalEvent>,

    pub telemetry_tx: AsyncSender<TelemetryEvent>,
    pub telemetry_rx: AsyncReceiver<TelemetryEvent>,

    pub lifecycle_tx: AsyncSender<LifecycleEvent>,
    pub lifecycle_rx: AsyncReceiver<LifecycleEvent>,

    pub diagnostic_tx: AsyncSender<DiagnosticEvent>,
    pub diagnostic_rx: AsyncReceiver<DiagnosticEvent>,
}

impl Bus {
    #[must_use]
    pub fn new(
        critical_capacity: usize,
        telemetry_capacity: usize,
        lifecycle_capacity: usize,
        diagnostic_capacity: usize,
    ) -> Self {
        let (critical_tx, _) =
            broadcast::channel(critical_capacity);

        let (telemetry_tx, telemetry_rx) =
            kanal::bounded_async(telemetry_capacity);

        let (lifecycle_tx, lifecycle_rx) =
            kanal::bounded_async(lifecycle_capacity);

        let (diagnostic_tx, diagnostic_rx) =
            kanal::bounded_async(diagnostic_capacity);

        Self {
            critical_tx,

            telemetry_tx,
            telemetry_rx,

            lifecycle_tx,
            lifecycle_rx,

            diagnostic_tx,
            diagnostic_rx,
        }
    }

    #[must_use]
    pub fn subscribe_critical(
        &self,
    ) -> broadcast::Receiver<CriticalEvent> {
        self.critical_tx.subscribe()
    }

    #[must_use]
    pub fn telemetry_receiver(
        &self,
    ) -> AsyncReceiver<TelemetryEvent> {
        self.telemetry_rx.clone()
    }

    #[must_use]
    pub fn lifecycle_receiver(
        &self,
    ) -> AsyncReceiver<LifecycleEvent> {
        self.lifecycle_rx.clone()
    }

    #[must_use]
    pub fn diagnostic_receiver(
        &self,
    ) -> AsyncReceiver<DiagnosticEvent> {
        self.diagnostic_rx.clone()
    }

    pub fn emit_critical(
        &self,
        event: CriticalEvent,
    ) -> Result<
        usize,
        broadcast::error::SendError<CriticalEvent>,
    > {
        self.critical_tx.send(event)
    }

    pub async fn emit_telemetry(
        &self,
        event: TelemetryEvent,
    ) {
        let _ = self.telemetry_tx.send(event).await;
    }

    pub async fn emit_lifecycle(
        &self,
        event: LifecycleEvent,
    ) {
        let _ = self.lifecycle_tx.send(event).await;
    }

    pub fn emit_diagnostic(
        &self,
        event: DiagnosticEvent,
    ) {
        let _ = self.diagnostic_tx.try_send(event);
    }
}