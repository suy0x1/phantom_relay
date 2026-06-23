use crate::monitor::level::Level;
use serde::{Deserialize, Serialize};

/// Configuration of the logger
///
/// Contorls the logging level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde[default]]
pub struct LoggerConfig {
    pub level: Level,
}

impl LoggerConfig {
    pub fn default() -> Self {
        Default::default()
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: Level::ERROR,
        }
    }
}
