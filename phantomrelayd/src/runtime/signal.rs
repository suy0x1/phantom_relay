use anyhow::Result;

pub async fn wait_for_shutdown() -> Result<()> {
    tokio::signal::ctrl_c().await?;

    Ok(())
}
