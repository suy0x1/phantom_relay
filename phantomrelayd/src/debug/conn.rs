use anyhow::Result;
use std::sync::Arc;

use crate::routing::manager::ConnectionManager;

pub fn debug_conn(map: Arc<ConnectionManager>) -> Result<String> {
    Ok(format!("{:#?}", map))
}
