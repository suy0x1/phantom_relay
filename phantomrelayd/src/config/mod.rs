pub mod collector;
pub mod dns;
pub mod proxy;
pub mod rotation;
pub mod tproxy;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use collector::CollectorConfig;
use dns::DNSConfig;
use proxy::ProxyConfig;
use rotation::RotationConfig;
use tproxy::TProxyConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub collector: CollectorConfig,
    pub dns: DNSConfig,
    pub proxy: ProxyConfig,
    pub rotation: RotationConfig,
    pub tproxy: TProxyConfig,
}

impl Config {
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

    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}
