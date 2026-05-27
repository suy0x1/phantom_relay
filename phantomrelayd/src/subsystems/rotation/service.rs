use crate::monitor::events::CriticalEvent;
use crate::{config::rotation::RotationConfig, monitor::bus::Bus};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::Duration;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

/// Periodically emits proxy rotation signals on configured interval. Used to cycle through available proxies.
pub async fn start_rotating(
    config: Arc<RotationConfig>,
    bus: Arc<Bus>,
    cancel: CancellationToken,
) -> Result<()> {
    let mut ticker = interval(Duration::from_secs(config.rotate_sec));

    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                return Ok(());
            }

            _ = ticker.tick() => {

                bus.emit_critical(CriticalEvent::RotateProxy)?;
            }
        }
    }
}
