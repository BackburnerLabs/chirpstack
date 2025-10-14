//! Integration for passing data locally within chirpstack
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use tokio::sync::{broadcast, Mutex, MutexGuard, OnceCell};
use tokio_stream::wrappers::BroadcastStream;
use tracing::info;

use super::Integration as IntegrationTrait;
use chirpstack_api::integration;

static LOCAL_BACKEND: OnceCell<Mutex<LocalIntegrationBackend>> = OnceCell::const_new();

pub struct Integration {}

impl Integration {
    pub fn new() -> Integration {
        info!("Initializing Local integration");
        LOCAL_BACKEND
            .set(Mutex::new(LocalIntegrationBackend::new()))
            .map_err(|_| "Could not set LOCAL_BACKEND")
            .unwrap();
        Integration {}
    }
}

async fn get_backend() -> Result<MutexGuard<'static, LocalIntegrationBackend>> {
    Ok(LOCAL_BACKEND
        .get()
        .ok_or_else(|| anyhow!("LOCAL_BACKEND not set"))?
        .lock()
        .await)
}

/// Get stream of all uplink streams
pub async fn get_uplink_event_stream() -> Result<impl Stream<Item = integration::UplinkEvent>> {
    Ok(get_backend().await?.get_uplink_stream())
}

#[async_trait]
impl IntegrationTrait for Integration {
    async fn uplink_event(
        &self,
        _vars: &HashMap<String, String>,
        pl: &integration::UplinkEvent,
    ) -> Result<()> {
        get_backend().await?.log_uplink(pl)
    }

    async fn join_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::JoinEvent,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn ack_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::AckEvent,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn txack_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::TxAckEvent,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn log_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::LogEvent,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn status_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::StatusEvent,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn location_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::LocationEvent,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn integration_event(
        &self,
        _vars: &HashMap<String, String>,
        _pl: &integration::IntegrationEvent,
    ) -> Result<()> {
        unimplemented!()
    }
}

struct LocalIntegrationBackend {
    uplink_tx: broadcast::Sender<integration::UplinkEvent>,
}
impl LocalIntegrationBackend {
    fn new() -> Self {
        let (uplink_tx, _) = broadcast::channel(5);
        Self { uplink_tx }
    }

    fn get_uplink_stream(&self) -> impl Stream<Item = integration::UplinkEvent> {
        BroadcastStream::new(self.uplink_tx.subscribe()).filter_map(|v| async {
            if let Ok(ev) = v {
                Some(ev)
            } else {
                /* We don't care about lag errors - not much we can do about it*/
                None
            }
        })
    }

    fn log_uplink(&self, pl: &integration::UplinkEvent) -> Result<()> {
        self.uplink_tx.send(pl.clone())?;

        Ok(())
    }
}
