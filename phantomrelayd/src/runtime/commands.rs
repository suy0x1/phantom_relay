use crate::runtime::service::{Mode, Service};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeCommands {
    Start(Service),
    Stop(Service),
    Restart(Service),
    Enable(Mode),
    Disable(Mode),
    Status,
    Shutdown,
}
