use serde::{Deserialize, Serialize};

use crate::runtime::commands::RuntimeCommands;
use crate::runtime::controller::ServiceStatus; 

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCRequest {
    Runtime(RuntimeCommands),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCResponse {
    Success { message: String },

    Status { services: Vec<ServiceStatus> },

    Error { message: String },
}
