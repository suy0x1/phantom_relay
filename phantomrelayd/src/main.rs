use anyhow::Result;
// use phantom_relay::monitor::bus::Bus;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
// use tokio::sync::Mutex;

// use phantom_relay::ipc::server::start_ipc_server;
// use phantom_relay::runtime::startup::startup;

use phantom_relay::rotation::service::async_health_check;

#[tokio::main]
async fn main() -> Result<()> {
    // let bus = Arc::new(Bus::new(128));

    // let runtime = Arc::new(Mutex::new(startup(bus.clone()).await?));

    // let ipc_runtime = runtime.clone();
    // tokio::spawn(async move {
    //     let _ = start_ipc_server(ipc_runtime).await;
    // });

    // tokio::signal::ctrl_c().await?;

    // runtime.lock().await.shutdown().await?;
    let progress = Arc::new(AtomicU32::new(0));
    async_health_check(progress.clone()).await?;
    println!("{:#?}",progress);
    Ok(())
}
