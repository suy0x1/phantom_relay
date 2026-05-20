use serde::{
    Deserialize,
    Serialize,
};

#[derive(
    Serialize,
    Deserialize,
    Debug,
)]
pub struct ServiceStatus {

    pub name: String,

    pub active: bool,
}