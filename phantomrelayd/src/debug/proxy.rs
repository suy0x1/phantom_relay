use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use reqwest::Client;

use crate::collector::manager::HealthyProxy;

pub fn debug_proxy(
    debug: Arc<DashMap<HealthyProxy, Client>>,
) -> Result<String> {

    Ok(format!("{:#?}",debug))
}
