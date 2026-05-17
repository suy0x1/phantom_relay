use anyhow::Result;
use std::sync::Arc;
use crate::monitor::bus::Bus;

use crate::system::nftables::cleanup;

pub fn shutdown(bus: Arc<Bus>) -> Result<()> {
    cleanup(bus)?;

    Ok(())
}
