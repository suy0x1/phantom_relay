use anyhow::Result;
use phantomrelayd::monitor::bus::Bus;
use std::sync::Arc;
use tokio::sync::Mutex;

use phantomrelayd::runtime::startup::startup;
use phantomrelayd::ipc::server::start_ipc_server;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = Arc::new(Bus::new(128));

    let runtime = Arc::new(Mutex::new(startup(bus.clone()).await?));

    let ipc_runtime = runtime.clone();
    tokio::spawn(async move {
        let _ = start_ipc_server(ipc_runtime).await;
    });

    tokio::signal::ctrl_c().await?;

    runtime.lock().await.shutdown().await?;

    Ok(())
}
