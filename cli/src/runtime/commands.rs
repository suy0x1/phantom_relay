use serde::{Deserialize, Serialize};

use super::service::{Mode, Service};

#[derive(Serialize, Deserialize, Debug)]
pub enum RuntimeCommands {
    Start(Service),
    Stop(Service),
    Restart(Service),
    Enable(Mode),
    Disable(Mode),
    Status,
    Shutdown,
}
