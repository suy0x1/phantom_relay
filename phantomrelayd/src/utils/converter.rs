use anyhow::Result;

use crate::runtime::commands::RuntimeCommands;
use crate::runtime::service::Service;

pub fn convert_start(c: &str) -> Result<RuntimeCommands> {
    match c {
        "logger" => Ok(RuntimeCommands::Start(Service::Logger)),
        "proxy_collector" => Ok(RuntimeCommands::Start(Service::ProxyCollector)),
        "dns" => Ok(RuntimeCommands::Start(Service::DNS)),
        "proxy_rotator" => Ok(RuntimeCommands::Start(Service::ProxyRotator)),
        "cache_cleaner" => Ok(RuntimeCommands::Start(Service::CacheCleaner)),
        "cache_preloader" => Ok(RuntimeCommands::Start(Service::CachePreloader)),
        "cache_refresher" => Ok(RuntimeCommands::Start(Service::CacheRefresher)),
        "tproxy" => Ok(RuntimeCommands::Start(Service::TProxy)),
        "proxy" => Ok(RuntimeCommands::Start(Service::Proxy)),
        "metrics" => Ok(RuntimeCommands::Start(Service::Metrics)),
        _ => Err(anyhow::anyhow!("unknown service"))
    }
}