use serde::{Serialize, Deserialize};

/// Configuration for the default state
/// 
/// Controls the services that should start on the start-up of the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultState {
    pub services: Vec<String>,
}

impl DefaultState {
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for DefaultState {
    fn default() -> Self {
        Self {
            services: vec!["logger".to_string(), "metrics".to_string()],
        }
    }
}