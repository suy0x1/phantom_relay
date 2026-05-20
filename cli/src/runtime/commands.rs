use serde::{
    Deserialize,
    Serialize,
};

use super::service::Service;

#[derive(
    Serialize,
    Deserialize,
    Debug,
)]
pub enum RuntimeCommands {

    Start(Service),

    Stop(Service),

    Restart(Service),

    Status,

    Shutdown,
}