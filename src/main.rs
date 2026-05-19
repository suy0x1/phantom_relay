use anyhow::Result;
use std::sync::Arc;
use phantom_relay::monitor::bus::Bus;

use phantom_relay::runtime::{shutdown::shutdown, signal::wait_for_shutdown, startup::startup};

#[tokio::main]
async fn main() -> Result<()> {
    let bus = Arc::new(Bus::new(128));

    startup(bus.clone()).await?;
    wait_for_shutdown().await?;
    shutdown(bus.clone())?;

    Ok(())

}
