use crate::debug::route::debug_route;
use crate::debug::{
    config::{DebugConfig, debug_config},
    conn::debug_conn,
    dns::{DebugDns, debug_dns},
    proxy::debug_proxy,
};
use crate::runtime::service::Debug;
use crate::subsystems::rotation::manager::RotationEngine;
use crate::{runtime::context::RuntimeContext, subsystems::network::manager::NetworkManager};
use anyhow::{Result, anyhow};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;

use super::commands::RuntimeCommands;
use super::service::{Mode, Service, ServiceHandle};

pub type ServiceFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

use crate::runtime::factories::{
    cleanup_service, collector_service, dns_service, logger_service, metrics_service,
    preload_service, proxy_service, refresh_service, rotator_service, tproxy_service,
};

pub type ServiceFn = Arc<dyn Fn(CancellationToken) -> ServiceFuture + Send + Sync>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub active: bool,
    pub is_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Status(Vec<ServiceStatus>),
    Config(String),
    Conn(String),
    DNS(String),
    Proxy(String),
    Route(String),
}
pub struct RuntimeController {
    pub ctx: Arc<RuntimeContext>,
    pub services: DashMap<String, ServiceHandle>,
    pub modes: DashMap<String, bool>,
    pub networkmanager: Arc<NetworkManager>,
    pub rotation_engine: Arc<RotationEngine>,
}

impl RuntimeController {
    /// Creates a new runtime controller with network and rotation managers.
    pub fn new(ctx: RuntimeContext, rotation_engine: Arc<RotationEngine>) -> Self {
        Self {
            networkmanager: Arc::new(NetworkManager::new(
                ctx.bus.clone(),
                ctx.dns_config.clone(),
                ctx.tproxy_config.clone(),
            )),
            rotation_engine: rotation_engine.clone(),
            ctx: Arc::new(ctx),
            services: DashMap::new(),
            modes: DashMap::new(),
        }
    }

    /// Spawns and registers a service task with cancellation token.
    fn start_service(&self, name: &str, service: ServiceFn) -> Result<Vec<ServiceStatus>> {
        if self.services.contains_key(name) {
            return Err(anyhow!("service already running"));
        }

        let cancel = CancellationToken::new();

        let cancel_clone = cancel.clone();

        let task = tokio::spawn(async move {
            if let Err(e) = service(cancel_clone).await {
                eprintln!("{}", e);
            }
        });

        self.services
            .insert(name.to_string(), ServiceHandle { task, cancel });

        Ok(vec![ServiceStatus {
            name: name.to_string(),
            active: true,
            is_mode: false,
        }])
    }

    /// Cancels and removes a service task from tracking.
    async fn stop_service(&self, name: &str) -> Result<Vec<ServiceStatus>> {
        let (_, handle) = self
            .services
            .remove(name)
            .ok_or_else(|| anyhow!("service not found"))?;

        handle.cancel.cancel();

        let _ = handle.task.await;

        Ok(vec![ServiceStatus {
            name: name.to_string(),
            active: false,
            is_mode: false,
        }])
    }

    /// Cancels and awaits all running services. Blocks until all shutdown.
    pub async fn shutdown(&self) -> Result<Vec<ServiceStatus>> {
        let services: Vec<String> = self
            .services
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for name in services {
            if let Some((_, handle)) = self.services.remove(&name) {
                handle.cancel.cancel();

                let _ = handle.task.await;
            }
        }

        Ok(vec![ServiceStatus {
            name: "all services".to_string(),
            active: false,
            is_mode: false,
        }])
    }

    /// Checks if a service or mode is currently running.
    pub fn is_running(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Returns status of all registered services and modes.
    pub fn list_services(&self) -> Vec<ServiceStatus> {
        let services = vec![
            "logger",
            "proxy_collector",
            "dns",
            "proxy_rotator",
            "cache_preloader",
            "cache_cleaner",
            "cache_refresher",
            "tproxy",
            "proxy",
            "metrics",
        ];

        let modes = vec!["dns-turbo"];

        let mut status = Vec::new();

        // normal runtime services
        for name in services {
            status.push(ServiceStatus {
                name: name.to_string(),
                active: self.services.contains_key(name),
                is_mode: false,
            });
        }

        // runtime modes (dns turbo etc.)
        for mode in modes {
            status.push(ServiceStatus {
                name: mode.to_string(),
                active: self.services.contains_key(mode),
                is_mode: true,
            });
        }

        status
    }

    /// Handles runtime commands: start, stop, restart services; enable/disable modes; shutdown.
    pub async fn handle_commands(&mut self, cmd: RuntimeCommands) -> Result<Response> {
        match cmd {
            RuntimeCommands::Start(service) => match service {
                Service::Logger => {
                    let x = self.start_service("logger", logger_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }

                Service::ProxyCollector => {
                    let x =
                        self.start_service("proxy_collector", collector_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }

                Service::DNS => {
                    if self.ctx.healthy_proxies.len() == 0 {
                        return Err(anyhow!("no proxies marked healthy"));
                    }
                    let x = self.start_service("dns", dns_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }

                Service::ProxyRotator => {
                    let x =
                        self.start_service("proxy_rotator", rotator_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }

                Service::CachePreloader => {
                    if self.is_running("dns") {
                        let x: Vec<ServiceStatus> = self
                            .start_service("cache_preloader", preload_service(self.ctx.clone()))?;
                        return Ok(Response::Status(x));
                    } else {
                        return Err(anyhow!("DNS service must be running"));
                    }
                }

                Service::CacheCleaner => {
                    if self.is_running("dns") {
                        if !self.modes.contains_key("dns-turbo") {
                            let x = self.start_service(
                                "cache_cleaner",
                                cleanup_service(self.ctx.clone()),
                            )?;
                            return Ok(Response::Status(x));
                        } else {
                            Err(anyhow!(
                                "cannot run cache cleaner while dns turbo is active"
                            ))
                        }
                    } else {
                        return Err(anyhow!("DNS service must be running"));
                    }
                }

                Service::CacheRefresher => {
                    if self.is_running("dns") {
                        let x = self
                            .start_service("cache_refresher", refresh_service(self.ctx.clone()))?;
                        return Ok(Response::Status(x));
                    } else {
                        return Err(anyhow!("DNS service must be running"));
                    }
                }

                Service::TProxy => {
                    if self.ctx.healthy_proxies.len() == 0 {
                        return Err(anyhow!("no proxies marked healthy"));
                    }
                    let x = self.start_service("tproxy", tproxy_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }

                Service::Proxy => {
                    if self.ctx.healthy_proxies.len() == 0 {
                        return Err(anyhow!("no proxies marked healthy"));
                    }
                    let x = self.start_service("proxy", proxy_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }

                Service::Metrics => {
                    let x = self.start_service("metrics", metrics_service(self.ctx.clone()))?;
                    return Ok(Response::Status(x));
                }
            },

            RuntimeCommands::Stop(service) => match service {
                Service::Logger => {
                    let x = self.stop_service("logger").await?;
                    return Ok(Response::Status(x));
                }

                Service::ProxyCollector => {
                    let x = self.stop_service("proxy_collector").await?;
                    return Ok(Response::Status(x));
                }

                Service::DNS => {
                    let x = self.stop_service("dns").await?;
                    return Ok(Response::Status(x));
                }

                Service::ProxyRotator => {
                    let x = self.stop_service("proxy_rotator").await?;
                    return Ok(Response::Status(x));
                }

                Service::CachePreloader => {
                    let x = self.stop_service("cache_preloader").await?;
                    return Ok(Response::Status(x));
                }

                Service::CacheCleaner => {
                    let x = self.stop_service("cache_cleaner").await?;
                    return Ok(Response::Status(x));
                }

                Service::CacheRefresher => {
                    let x = self.stop_service("cache_refresher").await?;
                    return Ok(Response::Status(x));
                }

                Service::TProxy => {
                    let x = self.stop_service("tproxy").await?;
                    return Ok(Response::Status(x));
                }

                Service::Proxy => {
                    let x = self.stop_service("proxy").await?;
                    return Ok(Response::Status(x));
                }

                Service::Metrics => {
                    let x = self.stop_service("metrics").await?;
                    return Ok(Response::Status(x));
                }
            },

            RuntimeCommands::Restart(service) => match service {
                Service::Logger => {
                    self.stop_service("logger").await?;
                    let mut x = self.start_service("logger", logger_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }

                Service::ProxyCollector => {
                    self.stop_service("proxy_collector").await?;
                    let mut x =
                        self.start_service("proxy_collector", collector_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }

                Service::DNS => {
                    if self.ctx.healthy_proxies.len() == 0 {
                        return Err(anyhow!("no proxies marked healthy"));
                    }
                    self.stop_service("dns").await?;
                    let mut x = self.start_service("dns", dns_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }

                Service::ProxyRotator => {
                    self.stop_service("proxy_rotator").await?;
                    let mut x =
                        self.start_service("proxy_rotator", rotator_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }

                Service::CachePreloader => {
                    if self.is_running("dns") {
                        self.stop_service("cache_preloader").await?;
                        let mut x = self
                            .start_service("cache_preloader", preload_service(self.ctx.clone()))?;
                        x[0].name = x[0].name.replace(" started", " restarted");
                        return Ok(Response::Status(x));
                    } else {
                        return Err(anyhow!("DNS service must be running"));
                    }
                }

                Service::CacheCleaner => {
                    if self.is_running("dns") {
                        if self.modes.contains_key("dns-turbo") {
                            self.stop_service("cache_cleaner").await?;
                            let mut x = self.start_service(
                                "cache_cleaner",
                                cleanup_service(self.ctx.clone()),
                            )?;
                            x[0].name = x[0].name.replace(" started", " restarted");
                            return Ok(Response::Status(x));
                        } else {
                            return Err(anyhow!(
                                "cannot run cache cleaner while dns turbo is active"
                            ));
                        }
                    } else {
                        return Err(anyhow!("DNS service must be running"));
                    }
                }

                Service::CacheRefresher => {
                    if self.is_running("dns") {
                        self.stop_service("cache_refresher").await?;
                        let mut x = self
                            .start_service("cache_refresher", refresh_service(self.ctx.clone()))?;
                        x[0].name = x[0].name.replace(" started", " restarted");
                        return Ok(Response::Status(x));
                    } else {
                        return Err(anyhow!("DNS service must be running"));
                    }
                }

                Service::TProxy => {
                    if self.ctx.healthy_proxies.len() == 0 {
                        return Err(anyhow!("no proxies marked healthy"));
                    }
                    self.stop_service("tproxy").await?;
                    let mut x = self.start_service("tproxy", tproxy_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }

                Service::Proxy => {
                    if self.ctx.healthy_proxies.len() == 0 {
                        return Err(anyhow!("no proxies marked healthy"));
                    }
                    self.stop_service("proxy").await?;
                    let mut x = self.start_service("proxy", proxy_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }

                Service::Metrics => {
                    self.stop_service("metrics").await?;
                    let mut x = self.start_service("metrics", metrics_service(self.ctx.clone()))?;
                    x[0].name = x[0].name.replace(" started", " restarted");
                    return Ok(Response::Status(x));
                }
            },

            RuntimeCommands::Enable(m) => match m {
                Mode::CacheReloader => {
                    if self.is_running("dns") && self.is_running("cache_refresher") {
                        if !self.services.contains_key("cache_cleaner") {
                            self.ctx.dns_config.lock().await.cache_saturation = true;
                            self.modes.insert("dns-turbo".to_string(), true);
                            return Ok(Response::Status(vec![ServiceStatus {
                                name: "turbo cache mode".to_string(),
                                active: true,
                                is_mode: true,
                            }]));
                        } else {
                            return Err(anyhow!(
                                "cannot enable dns turbo while cache cleaner is running"
                            ));
                        }
                    } else {
                        return Err(anyhow!("DNS and cache_refresher services must be running"));
                    }
                }
            },

            RuntimeCommands::Disable(m) => match m {
                Mode::CacheReloader => {
                    self.ctx.dns_config.lock().await.cache_saturation = false;
                    self.modes.remove(&"dns-turbo".to_string());
                    return Ok(Response::Status(vec![ServiceStatus {
                        name: "turbo cache mode".to_string(),
                        active: false,
                        is_mode: true,
                    }]));
                }
            },

            RuntimeCommands::Status => {
                let res = self.list_services();
                return Ok(Response::Status(res));
            }

            RuntimeCommands::Debug(d) => match d {
                Debug::Config => {
                    let x = DebugConfig::from_state(
                        &self.ctx.rotation_config,
                        &self.ctx.dns_config,
                        &self.ctx.tproxy_config,
                        &self.ctx.proxy_config,
                        &self.ctx.collector_config,
                    ).await;
                    Ok(Response::Config(debug_config(x)?))
                }

                Debug::Connection => Ok(Response::Conn(debug_conn(self.ctx.conn_map.clone())?)),

                Debug::DNS => {
                    let x = DebugDns::new(self.ctx.dns_cache.clone(), self.ctx.inflight.clone());
                    Ok(Response::DNS(debug_dns(x)?))
                }

                Debug::Proxy => Ok(Response::Proxy(debug_proxy(self.ctx.healthy_proxies.clone())?)),

                Debug::Route => Ok(Response::Route(debug_route(self.ctx.current_route.clone()).await?)),
            },
            RuntimeCommands::Shutdown => {
                let x = self.shutdown().await?;
                return Ok(Response::Status(x));
            }
        }
    }
}
