//! TF lookup / buffer errors.

use thiserror::Error;

/// Errors from TF buffer lookup and tree walks.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TfError {
    #[error("frame '{0}' is unknown")]
    UnknownFrame(String),

    #[error("no transform path from '{from_frame}' to '{to_frame}'")]
    Connectivity {
        to_frame: String,
        from_frame: String,
    },

    #[error("lookup '{target}' ← '{from_frame}' failed: {reason}")]
    Lookup {
        target: String,
        from_frame: String,
        reason: String,
    },

    #[error("invalid transform: {0}")]
    Invalid(String),
}

impl TfError {
    pub(crate) fn connectivity(target: impl Into<String>, source: impl Into<String>) -> Self {
        Self::Connectivity {
            to_frame: target.into(),
            from_frame: source.into(),
        }
    }
}
