use std::io;
use thiserror::Error;

use hardy_bpa::cla::Error as ClaError;

/// Result type for CLA operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors encountered by the Convergence Layer Adapter.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("CLA not registered with BPA")]
    NotRegistered,

    #[error("Driver channel closed")]
    DriverClosed,

    #[error("Connection timed out")]
    Timeout,

    #[error("Connection refused or aborted by peer")]
    ConnectionFailed,

    #[error("Session busy")]
    Busy,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid peer address: {0}")]
    InvalidAddress(String),

    #[error("Bundle buffer error: {0}")]
    Buffer(#[from] hardy_bpa::stream::BufferError),

    #[error("Bundle too large: size {size} exceeds maximum {max}")]
    PayloadTooLarge { size: u64, max: u64 },
}

impl From<Error> for ClaError {
    fn from(err: Error) -> Self {
        match err {
            Error::Buffer(e) => ClaError::from(e),
            Error::PayloadTooLarge { size, max } => ClaError::PayloadTooLarge { size, max },
            other => ClaError::Internal(Box::new(other)),
        }
    }
}
