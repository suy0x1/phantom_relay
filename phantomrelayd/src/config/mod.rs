/// Configuration for the collector subsystem.
pub mod collector;
/// Configuration for the default state
pub mod defaultstate;
/// Configuration for the DNS subsystem.
pub mod dns;
/// Configuration for the logger
pub mod logger;
/// Configuration for the proxy subsystem.
pub mod proxy;
/// Configuration for the rotation subsystem.
pub mod rotation;
/// Configuration for the transparent proxy subsystem.
pub mod tproxy;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use collector::CollectorConfig;
use defaultstate::DefaultState;
use dns::DNSConfig;
use logger::LoggerConfig;
use proxy::ProxyConfig;
use rotation::RotationConfig;
use tproxy::TProxyConfig;

/// Global configuration for phantom relay daemon.
///
/// Aggregates settings for all subsystems: collector, DNS, proxy, rotation, and transparent proxy.
/// Can be loaded from or saved to a TOML file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub collector: CollectorConfig,
    pub dns: DNSConfig,
    pub proxy: ProxyConfig,
    pub rotation: RotationConfig,
    pub tproxy: TProxyConfig,
    pub default: DefaultState,
    pub logger: LoggerConfig,
}

impl Config {
    /// Loads configuration from a TOML file, or creates a default if the file doesn't exist.
    ///
    /// # Arguments
    /// * `path` - File path to the configuration file.
    ///
    /// # Returns
    /// The loaded or newly created configuration.
    pub fn load_or_create(path: &str) -> Result<Self> {
        let p = Path::new(path);

        if !p.exists() {
            let default_cfg = Config::default();
            default_cfg.save(path)?;
            return Ok(default_cfg);
        }

        let raw = fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    /// Saves the configuration to a TOML file.
    ///
    /// # Arguments
    /// * `path` - File path where the configuration will be saved.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written or the configuration cannot be serialized.
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}
