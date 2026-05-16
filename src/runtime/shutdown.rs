use anyhow::Result;

use crate::system::nftables::cleanup;

pub fn shutdown() -> Result<()> {
    cleanup()?;

    Ok(())
}
