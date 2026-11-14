//! BPA Convergence Layer Adapter implementation for `template-dtnpm-cla`.

use std::num::{NonZeroU32, NonZeroU64};
use std::sync::Arc;

use tokio::sync::{RwLock, mpsc, oneshot};
use tracing::{debug, info};

use hardy_async::TaskPool;
use hardy_bpa::async_trait;
use hardy_bpa::cla::{
    Cla, ClaAddress, ClaInit, ForwardBundleResult, Result as ClaResult, Segment, Sink,
};
use hardy_bpa::stream::{Receiver, buffer_stream};
use hardy_bpv7::bundle::Id;
use hardy_bpv7::eid::NodeId;

use crate::config::{ClaConfig, PeerEntry};
use crate::driver::{Driver, DriverCommand, from_cla_address, to_cla_address};
use crate::error::{Error, Result};

struct ClaInner {
    sink: Arc<dyn Sink>,
    tasks: Arc<TaskPool>,
    cmd_tx: mpsc::Sender<DriverCommand>,
    _driver_handle: tokio::task::JoinHandle<()>,
}

/// Convergence Layer Adapter for `template-dtnpm-cla`.
#[derive(Clone)]
pub struct TemplateCla {
    config: ClaConfig,
    inner: Arc<RwLock<Option<ClaInner>>>,
}

impl TemplateCla {
    /// Create a new instance of [`TemplateCla`].
    #[must_use]
    pub fn new(config: ClaConfig) -> Self {
        Self {
            config,
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialisation parameters for BPA registration.
    #[must_use]
    pub fn cla_init(&self) -> ClaInit {
        ClaInit {
            address_type: None,
            lane_count: NonZeroU32::new(1),
            max_bundle_size: NonZeroU64::new(self.config.max_frame_size as u64),
        }
    }

    /// Registers a peer mapping dynamically.
    pub async fn register_peer(&self, peer: PeerEntry) -> Result<()> {
        let cmd_tx = {
            let inner_guard = self.inner.read().await;
            let inner = inner_guard.as_ref().ok_or(Error::NotRegistered)?;
            inner.cmd_tx.clone()
        };

        cmd_tx
            .send(DriverCommand::RegisterPeer { peer })
            .await
            .map_err(|_| Error::DriverClosed)
    }

    /// Disconnects any active transport session.
    pub async fn disconnect(&self) -> Result<()> {
        let cmd_tx = {
            let inner_guard = self.inner.read().await;
            let inner = inner_guard.as_ref().ok_or(Error::NotRegistered)?;
            inner.cmd_tx.clone()
        };

        let (reply_tx, reply_rx) = oneshot::channel();
        match tokio::time::timeout(std::time::Duration::from_secs(5), async {
            cmd_tx
                .send(DriverCommand::Disconnect { reply: reply_tx })
                .await
                .map_err(|_| Error::DriverClosed)?;
            reply_rx.await.map_err(|_| Error::DriverClosed)?
        })
        .await
        {
            Ok(res) => res,
            Err(_) => Err(Error::Timeout),
        }
    }

    /// Close connection associated with an address.
    pub async fn close(&self, cla_addr: &ClaAddress) {
        debug!(cla_address = ?cla_addr, "Close requested for CLA address");
    }

    /// Resolve a DTN NodeId to a local CLA address.
    pub async fn peer_resolver(&self, peer: &NodeId) -> Option<ClaAddress> {
        let peer_str = peer.to_string();
        self.config
            .peers
            .get(&peer_str)
            .map(|entry| to_cla_address(&entry.address.to_string()))
    }

    /// Unregisters this CLA from the BPA and shuts down background tasks.
    pub async fn unregister(&self) {
        let mut inner_guard = self.inner.write().await;
        if let Some(inner) = inner_guard.take() {
            inner.sink.unregister().await;
            let _ = inner.cmd_tx.send(DriverCommand::Shutdown).await;
            let _ = inner._driver_handle.await;
            inner.tasks.shutdown().await;
        }
    }
}

#[async_trait]
impl Cla for TemplateCla {
    async fn on_register(
        &self,
        sink: Box<dyn Sink>,
        _node_ids: &[NodeId],
        _max_bundle_size: Option<NonZeroU64>,
    ) {
        info!("Registering Template CLA with BPA");
        let sink: Arc<dyn Sink> = sink.into();
        let tasks = Arc::new(TaskPool::new());
        let (cmd_tx, cmd_rx) = mpsc::channel(64);

        let driver_handle = Driver::spawn(self.config.clone(), sink.clone(), tasks.clone(), cmd_rx);

        let mut inner_guard = self.inner.write().await;
        *inner_guard = Some(ClaInner {
            sink,
            tasks,
            cmd_tx,
            _driver_handle: driver_handle,
        });
    }

    async fn on_unregister(&self) {
        info!("Unregistering Template CLA from BPA");
        let mut inner_guard = self.inner.write().await;
        if let Some(inner) = inner_guard.take() {
            let _ = inner.cmd_tx.send(DriverCommand::Shutdown).await;
            let _ = inner._driver_handle.await;
            inner.tasks.shutdown().await;
        }
    }

    async fn forward(
        &self,
        _lane: Option<u32>,
        cla_addr: &ClaAddress,
        bundle_id: &Id,
        total_len: u64,
        stream: &mut dyn Receiver<Segment>,
    ) -> ClaResult<ForwardBundleResult> {
        let cmd_tx = {
            let inner_guard = self.inner.read().await;
            let inner = inner_guard
                .as_ref()
                .ok_or(hardy_bpa::cla::Error::Disconnected)?;
            inner.cmd_tx.clone()
        };

        if total_len > self.config.max_frame_size as u64 {
            return Err(hardy_bpa::cla::Error::from(Error::PayloadTooLarge {
                size: total_len,
                max: self.config.max_frame_size as u64,
            }));
        }

        let Some(target) = from_cla_address(cla_addr) else {
            return Ok(ForwardBundleResult::NoNeighbour);
        };

        let bundle_bytes = buffer_stream(stream, total_len)
            .await
            .map_err(Error::Buffer)?;

        debug!(
            bundle_id = %bundle_id,
            target = %target,
            bytes = bundle_bytes.len(),
            "Forwarding bundle frame via CLA"
        );

        let (reply_tx, reply_rx) = oneshot::channel();
        cmd_tx
            .send(DriverCommand::ForwardBundle {
                target,
                data: bundle_bytes,
                reply: reply_tx,
            })
            .await
            .map_err(|_| hardy_bpa::cla::Error::Disconnected)?;

        match reply_rx.await {
            Ok(Ok(res)) => Ok(res),
            Ok(Err(e)) => Err(hardy_bpa::cla::Error::from(e)),
            Err(_) => Err(hardy_bpa::cla::Error::Disconnected),
        }
    }
}
