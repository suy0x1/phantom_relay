use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::Result;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use tokio::sync::Mutex;

use crate::{
    config::{dns::DNSConfig, tproxy::TProxyConfig},
    monitor::bus::Bus,
    monitor::events::Event,
    runtime::context::RuntimeContext,
};

use super::{
    capablities::NetworkCapability,
    rules::{
        block_quic, ensure_base_stack, ignore_localhost, redirect_dns, redirect_tcp,
        remove_dns_redirect, remove_localhost_bypass, remove_table, remove_tcp_redirect,
        unblock_quic,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CapMeta {
    priority: u16,
    active: bool,
}
pub struct NetworkManager {
    pub enabled: DashMap<NetworkCapability, CapMeta>,
    pub initialized: AtomicBool,
    pub bus: Arc<Bus>,
    pub dns_config: Arc<Mutex<DNSConfig>>,
    pub tproxy_config: Arc<TProxyConfig>,
}

impl NetworkManager {
    pub fn new(
        bus: Arc<Bus>,
        dns_config: Arc<Mutex<DNSConfig>>,
        tproxy_config: Arc<TProxyConfig>,
    ) -> Self {
        Self {
            enabled: DashMap::new(),
            initialized: AtomicBool::new(false),
            bus,
            dns_config,
            tproxy_config,
        }
    }

    pub fn ensure_initialized(&self) -> Result<()> {
        if self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }

        ensure_base_stack(self.bus.clone())?;

        self.initialized.store(true, Ordering::Relaxed);

        Ok(())
    }

    pub async fn enable(&self, capability: NetworkCapability) -> Result<()> {

        self.ensure_initialized()?;

        match capability {
            NetworkCapability::QUICBlocking => {
                self.enabled.insert(
                    capability,
                    CapMeta {
                        priority: 0,
                        active: false,
                    },
                );
            }

            NetworkCapability::LocalhostBypass => {
                self.enabled.insert(
                    capability,
                    CapMeta {
                        priority: 1,
                        active: false,
                    },
                );
            }

            NetworkCapability::TransparentProxy => {
                self.enabled.insert(
                    capability,
                    CapMeta {
                        priority: 3,
                        active: false,
                    },
                );
            }

            NetworkCapability::DNSIntercept => {
                self.enabled.insert(
                    capability,
                    CapMeta {
                        priority: 2,
                        active: false,
                    },
                );
            }
        }
        let mut d: Vec<(NetworkCapability, CapMeta)> = self
            .enabled
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        d.sort_by_key(|(_, value)| *&value.priority);

        for (capb, _) in d {
            if let Some(entry) = self.enabled.get(&capb) {
                if entry.active {
                    continue;
                }
            }
            match capb {
                NetworkCapability::QUICBlocking => {
                    block_quic(self.bus.clone())?;
                }

                NetworkCapability::LocalhostBypass => {
                    ignore_localhost(self.bus.clone())?;
                }

                NetworkCapability::TransparentProxy => {
                    redirect_tcp(self.tproxy_config.clone(), self.bus.clone())?;
                }

                NetworkCapability::DNSIntercept => {
                    redirect_dns(self.dns_config.clone(), self.bus.clone()).await?;
                }
            }
            if let Some(mut entry) = self.enabled.get_mut(&capb) {
                entry.active = true;
            }
        }

        Ok(())
    }

    pub async fn disable(&self, capability: &NetworkCapability) -> Result<()> {
        if !self.enabled.contains_key(capability) {
            return Ok(());
        }

        match capability {
            NetworkCapability::QUICBlocking => {
                unblock_quic(self.bus.clone())?;
            }

            NetworkCapability::LocalhostBypass => {
                remove_localhost_bypass(self.bus.clone())?;
            }

            NetworkCapability::TransparentProxy => {
                remove_tcp_redirect(self.tproxy_config.clone(), self.bus.clone())?;
            }

            NetworkCapability::DNSIntercept => {
                remove_dns_redirect(self.dns_config.clone(), self.bus.clone()).await?;
            }
        }

        self.enabled.remove(capability);

        if self.enabled.is_empty() {
            remove_table(self.bus.clone())?;

            self.initialized.store(false, Ordering::Relaxed);
        }

        Ok(())
    }

    pub async fn network_sub(
        &self,
        ctx: Arc<RuntimeContext>,
        cancel: CancellationToken,
    ) -> Result<()> {
        let mut rx = ctx.bus.subscribe();

        self.ensure_initialized()?;

        loop {
            tokio::select! {

                _ = cancel.cancelled() => {
                    remove_table(ctx.bus.clone())?;
                    break;
                }

                msg = rx.recv() => {
                    match msg {

                        Ok(Event::EnableCapability { cap, timestamp: _ }) => {
                            self.enable(cap.clone()).await?;
                            }

                        Ok(Event::DisableCapability { cap, timestamp: _ }) => {
                            self.disable(&cap).await?;
                        }

                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    pub fn spawn_network_manager(
        self: Arc<Self>,
        ctx: Arc<RuntimeContext>,
        cancel: CancellationToken,
    ) {
        tokio::spawn(async move {
            let _ = self.network_sub(ctx, cancel).await;
        });
    }

    pub fn cleanup_all(&self) -> Result<()> {
        self.enabled.clear();

        remove_table(self.bus.clone())?;

        self.initialized.store(false, Ordering::Relaxed);

        Ok(())
    }
}
