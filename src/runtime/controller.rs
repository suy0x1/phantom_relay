use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio_util::sync::CancellationToken;
use super::service::ServiceHandle;

pub type ServiceFuture =
    Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub type ServiceFn =
    Arc<dyn Fn(CancellationToken) -> ServiceFuture + Send + Sync>;

pub struct RuntimeController {
    pub services: DashMap<String, ServiceHandle>,
}

impl RuntimeController {
    pub fn new() -> Self {
        Self {
            services: DashMap::new(),
        }
    }

    pub fn start_service(
        &self,
        name: &str,
        service: ServiceFn,
    ) -> Result<()> {

        if self.services.contains_key(name) {
            return Err(anyhow!(
                "service already running"
            ));
        }

        let cancel =
            CancellationToken::new();

        let cancel_clone =
            cancel.clone();

        let task = tokio::spawn(async move {
            if let Err(e) =
                service(cancel_clone).await
            {
                eprintln!("{}", e);
            }
        });

        self.services.insert(
            name.to_string(),
            ServiceHandle {
                task,
                cancel,
            },
        );

        Ok(())
    }

    pub async fn stop_service(
        &self,
        name: &str,
    ) -> Result<()> {

        let (_, handle) =
            self.services
                .remove(name)
                .ok_or_else(|| {
                    anyhow!("service not found")
                })?;

        handle.cancel.cancel();

        let _ = handle.task.await;

        Ok(())
    }

    pub fn is_running(
        &self,
        name: &str,
    ) -> bool {
        self.services.contains_key(name)
    }

    pub fn list_services(&self) -> Vec<String> {
        self.services
            .iter()
            .map(|x| x.key().clone())
            .collect()
    }
}