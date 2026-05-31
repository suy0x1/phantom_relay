use serde::{Deserialize, Serialize};

use crate::runtime::commands::RuntimeCommands;
use crate::runtime::controller::ServiceStatus;

/// Request message sent over IPC to the daemon.
///
/// Encapsulates runtime commands to control services and modes.
#[derive(Serialize, Deserialize, Debug)]
pub enum IPCRequest {
    /// Runtime command (start, stop, restart services or modes).
    Runtime(RuntimeCommands),
}

/// Response message sent from the daemon over IPC.
///
/// Indicates success, failure, or returns service status information.
#[derive(Serialize, Deserialize, Debug)]
pub enum IPCResponse {
    /// Successful operation completion.
    Success {
        /// Human-readable success message.
        message: String,
    },

    /// Status of all services and modes.
    Status {
        /// Current status of each registered service and mode.
        services: Vec<ServiceStatus>,
    },

    /// Operation failed with an error.
    Error {
        /// Human-readable error message describing what went wrong.
        message: String,
    },
}
