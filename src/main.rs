use anyhow::Result;

use phantom_relay::runtime::{shutdown::shutdown, signal::wait_for_shutdown, startup::startup};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting phantom-relay...");

    startup(9001, 9002).await?;
    wait_for_shutdown().await?;
    shutdown()?;

    Ok(())

}
