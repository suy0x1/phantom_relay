use anyhow::{Result, anyhow};

use crate::{
    cli::args::Commands,
    runtime::{
        commands::RuntimeCommands,
        service::{Mode, Service},
    },
};

/// Parses a service name string into the corresponding Service variant.
fn parse_service(service: &str) -> Result<Service> {
    match service {
        "logger" => Ok(Service::Logger),

        "proxy-collector" => Ok(Service::ProxyCollector),

        "dns" => Ok(Service::DNS),

        "rotator" => Ok(Service::ProxyRotator),

        "cache-cleaner" => Ok(Service::CacheCleaner),

        "cache-preloader" => Ok(Service::CachePreloader),

        "cache-refresher" => Ok(Service::CacheRefresher),

        "tproxy" => Ok(Service::TProxy),

        "proxy" => Ok(Service::Proxy),

        "metrics" => Ok(Service::Metrics),

        _ => Err(anyhow!("unknown service")),
    }
}

/// Parses a mode name string into the corresponding Mode variant.
fn parse_mode(mode: &str) -> Result<Mode> {
    match mode {
        "turbo-dns" => Ok(Mode::CacheReloader),

        _ => Err(anyhow!("unknown mode")),
    }
}

/// Converts CLI command arguments to runtime command protocol messages.
pub fn to_runtime_command(cmd: Commands) -> Result<RuntimeCommands> {
    match cmd {
        Commands::Start { service } => Ok(RuntimeCommands::Start(parse_service(&service)?)),

        Commands::Stop { service } => Ok(RuntimeCommands::Stop(parse_service(&service)?)),

        Commands::Restart { service } => Ok(RuntimeCommands::Restart(parse_service(&service)?)),

        Commands::Enable { mode } => Ok(RuntimeCommands::Enable(parse_mode(&mode)?)),

        Commands::Disable { mode } => Ok(RuntimeCommands::Disable(parse_mode(&mode)?)),

        Commands::Status => Ok(RuntimeCommands::Status),

        Commands::Shutdown => Ok(RuntimeCommands::Shutdown),
    }
}
