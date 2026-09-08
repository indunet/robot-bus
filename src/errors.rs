//! Broker-side error body conventions.

use thiserror::Error;

/// Base error type for the robot-bus SDK.
#[derive(Debug, Error)]
pub enum BusError {
    #[error("busy '{name}': callback queue is full")]
    Busy { name: String },

    #[error("handler panicked for '{name}'")]
    HandlerPanicked { name: String },

    #[error("no worker for '{name}'")]
    NoWorker { name: String },

    #[error("worker died for '{name}'")]
    WorkerDied { name: String },

    #[error("cancelled '{name}'")]
    Cancelled { name: String },

    #[error("action aborted: {0}")]
    ActionAborted(String),

    #[error("action rejected: {0}")]
    ActionRejected(String),

    #[error("no goal '{goal_id}'")]
    NoGoal { goal_id: String },

    #[error("{0}")]
    Timeout(String),

    #[error("{0}")]
    Protocol(String),

    #[error("Executor is closed")]
    Closed,

    #[error("parameter '{name}' is already declared")]
    ParameterAlreadyDeclared { name: String },

    #[error("parameter '{name}' is not declared")]
    ParameterNotDeclared { name: String },

    #[error("parameter '{name}' type mismatch: expected {expected}, got {got}")]
    ParameterTypeMismatch {
        name: String,
        expected: &'static str,
        got: &'static str,
    },

    #[error("parameter yaml: {0}")]
    ParameterYaml(String),

    #[error("zmq error: {0}")]
    Zmq(#[from] zmq::Error),
}

pub type Result<T> = std::result::Result<T, BusError>;

/// Map broker error prefixes to typed errors.
pub fn parse_error_body(body: &[u8]) -> Option<BusError> {
    if let Some(message) = strip_prefix(body, b"ACTION_ABORTED") {
        return Some(BusError::ActionAborted(decode_field(message)));
    }
    if let Some(message) = strip_prefix(body, b"ACTION_REJECTED") {
        return Some(BusError::ActionRejected(decode_field(message)));
    }
    if let Some(message) = strip_prefix(body, b"RPC_TIMEOUT") {
        return Some(BusError::Timeout(decode_field(message)));
    }
    if let Some(message) = strip_prefix(body, b"RPC_FAILED") {
        return Some(BusError::Protocol(decode_field(message)));
    }
    if let Some(name) = strip_prefix(body, b"BUSY") {
        return Some(BusError::Busy {
            name: decode_field(name),
        });
    }
    if let Some(name) = strip_prefix(body, b"HANDLER_PANICKED") {
        return Some(BusError::HandlerPanicked {
            name: decode_field(name),
        });
    }
    if let Some(name) = strip_prefix(body, b"NO_WORKER") {
        return Some(BusError::NoWorker {
            name: decode_field(name),
        });
    }
    if let Some(name) = strip_prefix(body, b"WORKER_DIED") {
        return Some(BusError::WorkerDied {
            name: decode_field(name),
        });
    }
    if let Some(name) = strip_prefix(body, b"CANCELLED") {
        return Some(BusError::Cancelled {
            name: decode_field(name),
        });
    }
    if let Some(goal_id) = strip_prefix(body, b"NO_GOAL") {
        return Some(BusError::NoGoal {
            goal_id: decode_field(goal_id),
        });
    }
    None
}

/// Encode a bridge RPC failure using the bus RESULT error convention.
pub fn rpc_error_body(error: &BusError) -> Vec<u8> {
    let (prefix, message) = match error {
        BusError::ActionAborted(message) => ("ACTION_ABORTED", message.clone()),
        BusError::ActionRejected(message) => ("ACTION_REJECTED", message.clone()),
        BusError::Cancelled { name } => ("CANCELLED", name.clone()),
        BusError::Timeout(message) => ("RPC_TIMEOUT", message.clone()),
        other => ("RPC_FAILED", other.to_string()),
    };
    format!("{prefix}\0{message}").into_bytes()
}

fn strip_prefix<'a>(body: &'a [u8], prefix: &[u8]) -> Option<&'a [u8]> {
    body.strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix(b"\0"))
}

fn decode_field(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
