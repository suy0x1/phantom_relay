use anyhow::Result;
use phantomrelayd::monitor::bus::Bus;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Mutex;

use phantomrelayd::ipc::server::start_ipc_server;
use phantomrelayd::runtime::startup::startup;

#[tokio::main]
async fn main() -> Result<()> {
    let bus = Arc::new(Bus::new(128, 1024, 1024, 1024));

    let runtime = Arc::new(Mutex::new(startup(bus.clone()).await?));

    let ipc_runtime = runtime.clone();
    tokio::spawn(async move {
        let _ = start_ipc_server(ipc_runtime).await;
    });

    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {runtime.lock().await.shutdown().await?;}
        _ = sigterm.recv() => {runtime.lock().await.shutdown().await?;}
    }

    Ok(())
}
