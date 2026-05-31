use anyhow::Result;
use kanal::{AsyncReceiver, AsyncSender};
use tokio::sync::broadcast;

use crate::monitor::events::{CriticalEvent, DiagnosticEvent, LifecycleEvent, TelemetryEvent};

/// Event bus for publishing and subscribing to system events.
///
/// Provides separate channels for critical events (broadcast), telemetry events (async queue),
/// lifecycle events (broadcast), and diagnostic events (broadcast). Each channel type has its own
/// capacity and subscription model.
#[derive(Clone)]
pub struct Bus {
    critical_tx: broadcast::Sender<CriticalEvent>,
    lifecycle_tx: broadcast::Sender<LifecycleEvent>,
    diagnostic_tx: broadcast::Sender<DiagnosticEvent>,

    telemetry_tx: AsyncSender<TelemetryEvent>,
    telemetry_rx: AsyncReceiver<TelemetryEvent>,
}

impl Bus {
    /// Creates a new event bus with the specified channel capacities.
    ///
    /// # Arguments
    /// * `critical_capacity` - Max messages in the critical event broadcast channel.
    /// * `telemetry_capacity` - Max messages in the telemetry event queue.
    /// * `lifecycle_capacity` - Max messages in the lifecycle event broadcast channel.
    /// * `diagnostic_capacity` - Max messages in the diagnostic event broadcast channel.
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

    /// Subscribes to critical events.
    ///
    /// # Returns
    /// A receiver for critical events. New subscribers receive only events published after subscription.
    #[must_use]
    pub fn subscribe_critical(&self) -> broadcast::Receiver<CriticalEvent> {
        self.critical_tx.subscribe()
    }

    /// Subscribes to lifecycle events.
    ///
    /// # Returns
    /// A receiver for lifecycle events. New subscribers receive only events published after subscription.
    #[must_use]
    pub fn subscribe_lifecycle(&self) -> broadcast::Receiver<LifecycleEvent> {
        self.lifecycle_tx.subscribe()
    }

    /// Subscribes to diagnostic events.
    ///
    /// # Returns
    /// A receiver for diagnostic events. New subscribers receive only events published after subscription.
    #[must_use]
    pub fn subscribe_diagnostic(&self) -> broadcast::Receiver<DiagnosticEvent> {
        self.diagnostic_tx.subscribe()
    }

    /// Gets a receiver for telemetry events.
    ///
    /// # Returns
    /// A receiver for telemetry events, allowing multiple clones to receive from the same queue.
    #[must_use]
    pub fn telemetry_receiver(&self) -> AsyncReceiver<TelemetryEvent> {
        self.telemetry_rx.clone()
    }

    /// Publishes a critical event to all subscribers.
    ///
    /// # Arguments
    /// * `event` - The critical event to publish.
    ///
    /// # Returns
    /// The number of subscribers that received the event, or a send error if the channel is closed.
    pub fn emit_critical(
        &self,
        event: CriticalEvent,
    ) -> Result<usize, broadcast::error::SendError<CriticalEvent>> {
        self.critical_tx.send(event)
    }

    /// Publishes a lifecycle event to all subscribers.
    ///
    /// # Arguments
    /// * `event` - The lifecycle event to publish.
    ///
    /// # Returns
    /// The number of subscribers that received the event, or a send error if the channel is closed.
    pub fn emit_lifecycle(
        &self,
        event: LifecycleEvent,
    ) -> Result<usize, broadcast::error::SendError<LifecycleEvent>> {
        self.lifecycle_tx.send(event)
    }

    /// Publishes a diagnostic event to all subscribers.
    ///
    /// # Arguments
    /// * `event` - The diagnostic event to publish.
    ///
    /// # Returns
    /// The number of subscribers that received the event, or a send error if the channel is closed.
    pub fn emit_diagnostic(
        &self,
        event: DiagnosticEvent,
    ) -> Result<usize, broadcast::error::SendError<DiagnosticEvent>> {
        self.diagnostic_tx.send(event)
    }

    /// Publishes a telemetry event to the queue (non-blocking).
    ///
    /// # Arguments
    /// * `event` - The telemetry event to publish.
    pub async fn emit_telemetry(&self, event: TelemetryEvent) {
        let _ = self.telemetry_tx.send(event).await;
    }
}
