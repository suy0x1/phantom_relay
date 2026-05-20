use serde::{Deserialize, Serialize};
use crate::runtime::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeCommands {
    Start(Service),
    Stop(Service),
    Restart(Service),
    Status,
    Shutdown,
}