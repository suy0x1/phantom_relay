use serde::{Deserialize, Serialize};

use crate::runtime::commands::RuntimeCommands;
use crate::runtime::status::Response;

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCRequest {
    Runtime(RuntimeCommands),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCResponse {
    Success { message: String },

    Status { services: Response },

    Error { message: String },
}
