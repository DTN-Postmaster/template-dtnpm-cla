use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Default maximum frame size in bytes (64 KiB).
pub const DEFAULT_MAX_FRAME: usize = 65_535;

/// Static peer mapping configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerEntry {
    /// Remote node DTN identifier (e.g. "dtn://station-b/")
    pub node_id: String,
    /// Network address of the remote CLA endpoint (e.g. "127.0.0.1:4556")
    pub address: SocketAddr,
}

/// Convergence Layer Adapter configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaConfig {
    /// Local socket address to bind to (e.g. "127.0.0.1:4556")
    pub bind_address: Option<SocketAddr>,

    /// Maximum frame size supported by this CLA instance (bytes)
    #[serde(default = "default_max_frame")]
    pub max_frame_size: usize,

    /// Connection attempt timeout
    #[serde(default = "default_connect_timeout", with = "humantime_duration")]
    pub connect_timeout: Duration,

    /// Explicit opt-in flag to allow unauthenticated or unsigned traffic
    #[serde(default)]
    pub allow_unauthenticated_opt_in: bool,

    /// Static peer lookup table mapping node IDs to peer definitions
    #[serde(default)]
    pub peers: HashMap<String, PeerEntry>,
}

fn default_max_frame() -> usize {
    DEFAULT_MAX_FRAME
}

fn default_connect_timeout() -> Duration {
    Duration::from_secs(10)
}

impl Default for ClaConfig {
    fn default() -> Self {
        Self {
            bind_address: None,
            max_frame_size: DEFAULT_MAX_FRAME,
            connect_timeout: Duration::from_secs(10),
            allow_unauthenticated_opt_in: false,
            peers: HashMap::new(),
        }
    }
}

impl ClaConfig {
    /// Validate configuration invariants.
    pub fn validate(&self) -> Result<()> {
        if self.max_frame_size < 128 {
            return Err(Error::InvalidConfig(
                "max_frame_size must be at least 128 bytes".to_string(),
            ));
        }
        if self.max_frame_size > 16 * 1024 * 1024 {
            return Err(Error::InvalidConfig(
                "max_frame_size cannot exceed 16 MiB".to_string(),
            ));
        }
        if self.connect_timeout.is_zero() {
            return Err(Error::InvalidConfig(
                "connect_timeout cannot be zero".to_string(),
            ));
        }
        Ok(())
    }
}

mod humantime_duration {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(
        duration: &Duration,
        s: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        s.collect_str(&humantime::format_duration(*duration))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Duration, D::Error> {
        let text = String::deserialize(d)?;
        humantime::parse_duration(&text).map_err(serde::de::Error::custom)
    }
}
