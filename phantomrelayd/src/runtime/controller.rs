use crate::{runtime::context::RuntimeContext, system::network::manager::NetworkManager};
use anyhow::{Result, anyhow};
use dashmap::DashMap;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;

use super::commands::RuntimeCommands;
use super::service::{Service, ServiceHandle};

pub type ServiceFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

use crate::runtime::factories::{
    cleanup_service, dns_service, logger_service, metrics_service, preload_service, proxy_service,
    refresh_service, tproxy_service,
};

pub type ServiceFn = Arc<dyn Fn(CancellationToken) -> ServiceFuture + Send + Sync>;

pub struct RuntimeController {
    pub ctx: Arc<RuntimeContext>,
    pub services: DashMap<String, ServiceHandle>,
    pub networkmanager: Arc<NetworkManager>,
}

impl RuntimeController {
    pub fn new(ctx: RuntimeContext) -> Self {
        Self {
            networkmanager: Arc::new(NetworkManager::new(
                ctx.bus.clone(),
                ctx.dns_config.clone(),
                ctx.tproxy_config.clone(),
            )),
            ctx: Arc::new(ctx),
            services: DashMap::new(),
        }
    }

    fn start_service(&self, name: &str, service: ServiceFn) -> Result<()> {
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

        Ok(())
    }

    async fn stop_service(&self, name: &str) -> Result<()> {
        let (_, handle) = self
            .services
            .remove(name)
            .ok_or_else(|| anyhow!("service not found"))?;

        handle.cancel.cancel();

        let _ = handle.task.await;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
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

        Ok(())
    }

    pub fn is_running(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    pub fn list_services(&self) {
        let x: Vec<String>  = self.services.iter().map(|x| x.key().clone()).collect();
        println!("{:#?}",x);
    }

    pub async fn handle_commands(&mut self, cmd: RuntimeCommands) -> Result<()> {
        match cmd {
            RuntimeCommands::Start(service) => match service {
                Service::Logger => {
                    self.start_service("logger", logger_service(self.ctx.clone()))?;
                }

                Service::DNS => {
                    self.start_service("dns", dns_service(self.ctx.clone()))?;
                }

                Service::CacheReloader => {
                    self.ctx.dns_config.lock().await.cache_saturation = true;
                }

                Service::CachePreloader => {
                    self.start_service("cache_preloader", preload_service(self.ctx.clone()))?;
                }

                Service::CacheCleaner => {
                    self.start_service("cache_cleaner", cleanup_service(self.ctx.clone()))?;
                }

                Service::CacheRefresher => {
                    self.start_service("cache_refresher", refresh_service(self.ctx.clone()))?;
                }

                Service::TProxy => {
                    self.start_service("tproxy", tproxy_service(self.ctx.clone()))?;
                }

                Service::Proxy => {
                    self.start_service("proxy", proxy_service(self.ctx.clone()))?;
                }

                Service::Metrics => {
                    self.start_service("metrics", metrics_service(self.ctx.clone()))?;
                }
            },

            RuntimeCommands::Stop(service) => match service {
                Service::Logger => {
                    self.stop_service("logger").await?;
                }

                Service::DNS => {
                    self.stop_service("dns").await?;
                }

                Service::CacheReloader => {
                    self.ctx.dns_config.lock().await.cache_saturation = false;
                }

                Service::CachePreloader => {
                    self.stop_service("cache_preloader").await?;
                }

                Service::CacheCleaner => {
                    self.start_service("cache_cleaner", cleanup_service(self.ctx.clone()))?;
                }

                Service::CacheRefresher => {
                    self.stop_service("cache_refresher").await?;
                }

                Service::TProxy => {
                    self.stop_service("tproxy").await?;
                }

                Service::Proxy => {
                    self.stop_service("proxy").await?;
                }

                Service::Metrics => {
                    self.stop_service("metrics").await?;
                }
            },

            RuntimeCommands::Restart(service) => match service {
                Service::Logger => {
                    self.stop_service("logger").await?;
                    self.start_service("logger", logger_service(self.ctx.clone()))?;
                }

                Service::DNS => {
                    self.stop_service("dns").await?;
                    self.start_service("dns", dns_service(self.ctx.clone()))?;
                }

                Service::CacheReloader => {}

                Service::CachePreloader => {
                    self.stop_service("cache_preloader").await?;
                    self.start_service("cache_preloader", preload_service(self.ctx.clone()))?;
                }

                Service::CacheCleaner => {
                    self.stop_service("cache_cleaner").await?;
                    self.start_service("cache_cleaner", cleanup_service(self.ctx.clone()))?;
                }

                Service::CacheRefresher => {
                    self.stop_service("cache_refresher").await?;
                    self.start_service("cache_refresher", refresh_service(self.ctx.clone()))?;
                }

                Service::TProxy => {
                    self.stop_service("tproxy").await?;
                    self.start_service("tproxy", tproxy_service(self.ctx.clone()))?;
                }

                Service::Proxy => {
                    self.stop_service("proxy").await?;
                    self.start_service("proxy", proxy_service(self.ctx.clone()))?;
                }

                Service::Metrics => {
                    self.stop_service("metrics").await?;
                    self.start_service("metrics", metrics_service(self.ctx.clone()))?;
                }
            },

            RuntimeCommands::Status => {
                self.list_services();
            }

            RuntimeCommands::Shutdown => {
                self.stop_all_services().await?;
            }
        }

        Ok(())
    }
}
