use serde::{Deserialize, Serialize};
use crate::runtime::service::{Service, Mode};

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