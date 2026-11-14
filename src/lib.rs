//! Template Convergence Layer Adapter (CLA) for DTN Postmaster.
//! Implements [`hardy_bpa::cla::Cla`].

pub mod cla;
pub mod config;
pub mod driver;
pub mod error;

pub use cla::TemplateCla;
pub use config::{ClaConfig, PeerEntry};
pub use driver::{Driver, DriverCommand, from_cla_address, to_cla_address};
pub use error::{Error, Result};
