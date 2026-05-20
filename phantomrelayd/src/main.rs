use anyhow::Result;
use phantom_relay::monitor::bus::Bus;
use std::sync::Arc;
use tokio::sync::Mutex;

use phantom_relay::runtime::startup::startup;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = Arc::new(Bus::new(128));

    let runtime = startup(bus.clone()).await?;

    // IPC server run

    tokio::signal::ctrl_c().await?;

    runtime.shutdown().await?;

    Ok(())
}
