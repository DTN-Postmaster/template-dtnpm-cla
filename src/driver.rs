//! Background driver managing network transport sessions and frame dispatching.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use hardy_async::TaskPool;
use hardy_bpa::cla::{Acceptance, ClaAddress, ForwardBundleResult, Sink};
use hardy_bpv7::eid::NodeId;

use crate::config::{ClaConfig, PeerEntry};
use crate::error::{Error, Result};

/// Converts a string target into a canonical [`ClaAddress`].
#[must_use]
pub fn to_cla_address(target: &str) -> ClaAddress {
    ClaAddress::Private(Bytes::from(target.as_bytes().to_vec()))
}

/// Extracts a target string from a [`ClaAddress`].
#[must_use]
pub fn from_cla_address(addr: &ClaAddress) -> Option<String> {
    match addr {
        ClaAddress::Private(bytes) => std::str::from_utf8(bytes).ok().map(|s| s.to_string()),
        _ => None,
    }
}

/// Commands sent to the background driver task.
pub enum DriverCommand {
    /// Forward a bundle to a specific peer endpoint.
    ForwardBundle {
        target: String,
        data: Bytes,
        reply: oneshot::Sender<Result<ForwardBundleResult>>,
    },
    /// Register a known peer mapping.
    RegisterPeer { peer: PeerEntry },
    /// Disconnect any active transport session.
    Disconnect { reply: oneshot::Sender<Result<()>> },
    /// Check if driver is currently running.
    Ping { reply: oneshot::Sender<bool> },
    /// Terminate driver loop.
    Shutdown,
}

/// State of an active peer session.
#[derive(Debug)]
pub struct Session {
    pub target: String,
    pub node_id: Option<NodeId>,
    pub last_activity: Instant,
}

/// Background worker managing transport loops and BPA dispatch.
pub struct Driver {
    config: ClaConfig,
    sink: Arc<dyn Sink>,
    tasks: Arc<TaskPool>,
    cmd_rx: mpsc::Receiver<DriverCommand>,
    peers: HashMap<String, PeerEntry>,
    active_sessions: HashMap<String, Session>,
}

impl Driver {
    /// Spawns the driver worker task.
    pub fn spawn(
        config: ClaConfig,
        sink: Arc<dyn Sink>,
        tasks: Arc<TaskPool>,
        cmd_rx: mpsc::Receiver<DriverCommand>,
    ) -> tokio::task::JoinHandle<()> {
        let mut driver = Self {
            peers: config.peers.clone(),
            config,
            sink,
            tasks,
            cmd_rx,
            active_sessions: HashMap::new(),
        };

        tokio::spawn(async move {
            driver.run().await;
        })
    }

    /// Access the BPA sink instance.
    pub fn sink(&self) -> &Arc<dyn Sink> {
        &self.sink
    }

    /// Access the driver's task pool.
    pub fn tasks(&self) -> &Arc<TaskPool> {
        &self.tasks
    }

    async fn run(&mut self) {
        info!("Driver loop started for template-dtnpm-cla");

        let mut ticker = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.prune_stale_sessions();
                }
                cmd = self.cmd_rx.recv() => {
                    let Some(command) = cmd else {
                        break;
                    };
                    if !self.handle_command(command).await {
                        break;
                    }
                }
            }
        }

        info!("Driver loop terminated for template-dtnpm-cla");
    }

    async fn handle_command(&mut self, command: DriverCommand) -> bool {
        match command {
            DriverCommand::ForwardBundle {
                target,
                data,
                reply,
            } => {
                let res = self.send_bundle(&target, data).await;
                let _ = reply.send(res);
                true
            }
            DriverCommand::RegisterPeer { peer } => {
                self.peers.insert(peer.node_id.clone(), peer);
                true
            }
            DriverCommand::Disconnect { reply } => {
                self.active_sessions.clear();
                let _ = reply.send(Ok(()));
                true
            }
            DriverCommand::Ping { reply } => {
                let _ = reply.send(true);
                true
            }
            DriverCommand::Shutdown => false,
        }
    }

    async fn send_bundle(&mut self, target: &str, data: Bytes) -> Result<ForwardBundleResult> {
        if data.len() > self.config.max_frame_size {
            return Err(Error::PayloadTooLarge {
                size: data.len() as u64,
                max: self.config.max_frame_size as u64,
            });
        }

        debug!(target = %target, bytes = data.len(), "Forwarding bundle frame via CLA driver");

        // Record or refresh session
        self.active_sessions.insert(
            target.to_string(),
            Session {
                target: target.to_string(),
                node_id: None,
                last_activity: Instant::now(),
            },
        );

        // In a real CLA, write bytes across network socket/serial device.
        // For the template mock/loopback driver, transmission succeeds.
        Ok(ForwardBundleResult::Sent)
    }

    /// Deliver an inbound bundle segment received from the transport to the BPA sink.
    pub async fn deliver_inbound(
        sink: &Arc<dyn Sink>,
        node_id: Option<&NodeId>,
        source_addr: &str,
        mut bundle_data: Bytes,
    ) -> Result<()> {
        let cla_addr = to_cla_address(source_addr);
        match sink
            .dispatch(node_id, Some(&cla_addr), &mut bundle_data)
            .await
        {
            Ok(Acceptance::Accepted) => {
                debug!("Inbound bundle delivered and accepted by BPA");
                Ok(())
            }
            Ok(Acceptance::Refused) => {
                warn!("Inbound bundle refused by BPA");
                Ok(())
            }
            Err(e) => {
                error!("Failed to dispatch bundle to BPA sink: {e}");
                Err(Error::Io(std::io::Error::other(e.to_string())))
            }
        }
    }

    fn prune_stale_sessions(&mut self) {
        let timeout = Duration::from_secs(300);
        let now = Instant::now();
        self.active_sessions
            .retain(|_, session| now.duration_since(session.last_activity) < timeout);
    }
}
