use crate::monitor::events::CriticalEvent;
use crate::{
    collector::manager::HealthyProxy, config::rotation::RotationConfig, monitor::bus::Bus,
};
use dashmap::DashMap;
use reqwest::Client;
use tokio::time::interval;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tokio::time::sleep;
use tokio::time::Duration;
use anyhow::Result;

pub async fn start_rotating(
    config: Arc<RotationConfig>,
    bus: Arc<Bus>,
    healthy_proxies: Arc<DashMap<HealthyProxy, Client>>,
    cancel: CancellationToken,
) -> Result<()>{
    loop {
        tokio::select! {

            _ = cancel.cancelled() => {
                return Ok(());
            }

            _ = sleep(Duration::from_secs(1)) => {

                if healthy_proxies.len() >= 1 {
                    bus.emit_critical(CriticalEvent::LoadInitialProxy)?;
                    break;
                }
            }
        }
    }

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
