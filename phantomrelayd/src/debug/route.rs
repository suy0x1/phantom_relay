use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use crate::subsystems::rotation::route::RouteContext;

pub async fn debug_route(debug: Arc<RwLock<RouteContext>>) -> Result<String> {
    Ok(format!("{:#?}", debug.read().await.clone()))
}
