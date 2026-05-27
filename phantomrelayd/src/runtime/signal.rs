use anyhow::Result;

/// Waits for CTRL-C signal to initiate graceful shutdown.
pub async fn wait_for_shutdown() -> Result<()> {
    tokio::signal::ctrl_c().await?;

    Ok(())
}
