use anyhow::anyhow;
use std::sync::Arc;
use std::thread::spawn;
use tokio::sync::{mpsc, RwLock};

use crate::app::dispatcher::Dispatcher;
use crate::app::dns_client::DnsClient;
use crate::app::inbound::manager::InboundManager;
use crate::app::nat_manager::NatManager;
use crate::app::stat_manager::StatManager;
use crate::app::{outbound::manager::OutboundManager, router::Router, SyncStatManager};
use crate::config::Config;
use crate::{new_runtime, sys, Error, Runner};

pub struct RelayManager {
    config: Arc<RwLock<Config>>,
    shutdown_tx: mpsc::Sender<()>,
    router: Arc<RwLock<Router>>,
    dns_client: Arc<RwLock<DnsClient>>,
    outbound_manager: Arc<RwLock<OutboundManager>>,
    #[cfg(feature = "stat")]
    stat_manager: SyncStatManager,
}

impl RelayManager {
    pub fn new(
        config: Arc<RwLock<Config>>,
        shutdown_tx: mpsc::Sender<()>,
        router: Arc<RwLock<Router>>,
        dns_client: Arc<RwLock<DnsClient>>,
        outbound_manager: Arc<RwLock<OutboundManager>>,
        #[cfg(feature = "stat")] stat_manager: SyncStatManager,
    ) -> Self {
        Self {
            config,
            shutdown_tx,
            router,
            dns_client,
            outbound_manager,
            #[cfg(feature = "stat")]
            stat_manager,
        }
    }

    #[cfg(feature = "stat")]
    pub fn stat_manager(&self) -> SyncStatManager {
        self.stat_manager.clone()
    }

    pub fn route_manager(&self) -> Arc<RwLock<Router>> {
        self.router.clone()
    }

    pub async fn set_outbound_selected(&self, outbound: &str, select: &str) -> Result<(), Error> {
        if let Some(selector) = self.outbound_manager.read().await.get_selector(outbound) {
            selector
                .write()
                .await
                .set_selected(select)
                .map_err(Error::Config)
        } else {
            Err(Error::Config(anyhow!("selector not found")))
        }
    }

    pub async fn get_outbound_selected(&self, outbound: &str) -> Result<String, Error> {
        if let Some(selector) = self.outbound_manager.read().await.get_selector(outbound) {
            return Ok(selector.read().await.get_selected_tag());
        }
        Err(Error::Config(anyhow!("not found")))
    }

    pub async fn get_outbound_selects(&self, outbound: &str) -> Result<Vec<String>, Error> {
        if let Some(selector) = self.outbound_manager.read().await.get_selector(outbound) {
            return Ok(selector.read().await.get_available_tags());
        }
        Err(Error::Config(anyhow!("not found")))
    }

    pub async fn update_config(&mut self, config: Config) -> Result<(), Error> {
        self.config = Arc::new(RwLock::new(config));
        Ok(())
    }

    // This function could block by an in-progress connection dialing.
    //
    // TODO Reload FakeDns. And perhaps the inbounds as long as the listening
    // addresses haven't changed.
    pub async fn reload(&self) -> Result<(), Error> {
        let mut config = self.config.read().await.clone();
        self.router.write().await.reload(&mut config.router)?;
        self.dns_client.write().await.reload(&config.dns)?;
        self.outbound_manager
            .write()
            .await
            .reload(&config.outbounds, self.dns_client.clone())
            .await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> bool {
        let tx = self.shutdown_tx.clone();
        if let Err(e) = tx.send(()).await {
            log::warn!("sending shutdown signal failed: {}", e);
            return false;
        }
        true
    }

    pub fn blocking_shutdown(&self) -> bool {
        let tx = self.shutdown_tx.clone();
        if let Err(e) = tx.blocking_send(()) {
            log::warn!("sending shutdown signal failed: {}", e);
            return false;
        }
        true
    }
}

pub fn create(mut config: Config) -> Result<RelayManager, Error> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    let rt = new_runtime(&crate::RuntimeOption::MultiThreadAuto(256 * 1024))?;
    let _g = rt.enter();

    let mut tasks: Vec<Runner> = Vec::new();
    let mut runners = Vec::new();

    let dns_client = Arc::new(RwLock::new(
        DnsClient::new(&config.dns).map_err(Error::Config)?,
    ));
    let outbound_manager = Arc::new(RwLock::new(
        OutboundManager::new(&config.outbounds, dns_client.clone()).map_err(Error::Config)?,
    ));
    let router = Arc::new(RwLock::new(Router::new(
        &mut config.router,
        dns_client.clone(),
    )));
    #[cfg(feature = "stat")]
    let stat_manager = Arc::new(RwLock::new(StatManager::new()));
    #[cfg(feature = "stat")]
    runners.push(StatManager::cleanup_task(stat_manager.clone()));
    let dispatcher = Arc::new(Dispatcher::new(
        outbound_manager.clone(),
        router.clone(),
        dns_client.clone(),
        #[cfg(feature = "stat")]
        stat_manager.clone(),
    ));

    let dispatcher_weak = Arc::downgrade(&dispatcher);
    let dns_client_cloned = dns_client.clone();

    let nat_manager = Arc::new(NatManager::new(dispatcher.clone()));
    let inbound_manager =
        InboundManager::new(&config.inbounds, dispatcher, nat_manager).map_err(Error::Config)?;
    let mut inbound_net_runners = inbound_manager
        .get_network_runners()
        .map_err(Error::Config)?;
    runners.append(&mut inbound_net_runners);

    #[cfg(all(feature = "inbound-tun", any(target_os = "macos", target_os = "linux")))]
    let net_info = if inbound_manager.has_tun_listener() && inbound_manager.tun_auto() {
        sys::get_net_info()
    } else {
        sys::NetInfo::default()
    };

    #[cfg(all(feature = "inbound-tun", any(target_os = "macos", target_os = "linux")))]
    {
        if let sys::NetInfo {
            default_interface: Some(iface),
            ..
        } = &net_info
        {
            let binds = if let Ok(v) = std::env::var("OUTBOUND_INTERFACE") {
                format!("{},{}", v, iface)
            } else {
                iface.clone()
            };
            std::env::set_var("OUTBOUND_INTERFACE", binds);
        }
    }

    #[cfg(all(
        feature = "inbound-tun",
        any(
            target_os = "ios",
            target_os = "android",
            target_os = "macos",
            target_os = "linux"
        )
    ))]
    if let Ok(r) = inbound_manager.get_tun_runner() {
        runners.push(r);
    }

    #[cfg(all(feature = "inbound-tun", any(target_os = "macos", target_os = "linux")))]
    sys::post_tun_creation_setup(&net_info);

    let rm = RelayManager::new(
        Arc::new(RwLock::new(config)),
        shutdown_tx,
        router,
        dns_client,
        outbound_manager,
        #[cfg(feature = "stat")]
        stat_manager,
    );

    // The main task joining all runners.
    tasks.push(Box::pin(async move {
        futures::future::join_all(runners).await;
    }));

    // Monitor shutdown signal.
    tasks.push(Box::pin(async move {
        let _ = shutdown_rx.recv().await;
    }));

    // Monitor ctrl-c exit signal.
    #[cfg(feature = "ctrlc")]
    tasks.push(Box::pin(async move {
        let _ = tokio::signal::ctrl_c().await;
    }));

    spawn(move || {
        rt.block_on(async move {
            dns_client_cloned
                .write()
                .await
                .replace_dispatcher(dispatcher_weak);
        });
        rt.block_on(futures::future::select_all(tasks));
        #[cfg(all(feature = "inbound-tun", any(target_os = "macos", target_os = "linux")))]
        sys::post_tun_completion_setup(&net_info);

        drop(inbound_manager);

        rt.shutdown_background();
    });

    Ok(rm)
}
