use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub active: bool,
    pub is_mode: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Status(Vec<ServiceStatus>),
    Config(String),
    Conn(String),
    DNS(String),
    Proxy(String),
    Route(String),
}
