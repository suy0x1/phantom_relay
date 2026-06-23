use serde::{Deserialize, Serialize};

use crate::runtime::service::Debug;

use super::service::{Mode, Service};

#[derive(Serialize, Deserialize, Debug)]
pub enum RuntimeCommands {
    Start(Service),
    Stop(Service),
    Restart(Service),
    Enable(Mode),
    Disable(Mode),
    Status,
    Debug(Debug),
    Metrics,
    Shutdown,
}
