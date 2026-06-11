use crate::runtime::service::{Debug, Mode, Service};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeCommands {
    Start(Service),
    Stop(Service),
    Restart(Service),
    Enable(Mode),
    Disable(Mode),
    Status,
    Debug(Debug),
    Shutdown,
}
