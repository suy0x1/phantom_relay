use anyhow::{
    anyhow,
    Result,
};

use crate::{
    cli::args::Commands,

    runtime::{
        commands::RuntimeCommands,
        service::Service,
    },
};

fn parse_service(
    service: &str,
) -> Result<Service> {

    match service {

        "logger" => {
            Ok(Service::Logger)
        }

        "dns" => {
            Ok(Service::DNS)
        }

        "cache-reloader" => {
            Ok(Service::CacheReloader)
        }

        "cache-cleaner" => {
            Ok(Service::CacheCleaner)
        }

        "cache-preloader" => {
            Ok(Service::CachePreloader)
        }

        "cache-refresher" => {
            Ok(Service::CacheRefresher)
        }

        "tproxy" => {
            Ok(Service::TProxy)
        }

        "proxy" => {
            Ok(Service::Proxy)
        }

        "metrics" => {
            Ok(Service::Metrics)
        }

        _ => {
            Err(anyhow!(
                "unknown service"
            ))
        }
    }
}

pub fn to_runtime_command(
    cmd: Commands,
) -> Result<RuntimeCommands> {

    match cmd {

        Commands::Start { service } => {

            Ok(
                RuntimeCommands::Start(
                    parse_service(&service)?
                )
            )
        }

        Commands::Stop { service } => {

            Ok(
                RuntimeCommands::Stop(
                    parse_service(&service)?
                )
            )
        }

        Commands::Restart { service } => {

            Ok(
                RuntimeCommands::Restart(
                    parse_service(&service)?
                )
            )
        }

        Commands::Status => {
            Ok(RuntimeCommands::Status)
        }

        Commands::Shutdown => {
            Ok(RuntimeCommands::Shutdown)
        }
    }
}