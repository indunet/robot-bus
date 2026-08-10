//! RPC status codes for the WebSocket gateway (wire-compatible with historical gRPC codes).

use std::fmt;

/// Status carried in TRAILER frames (`u32` code + message).
#[derive(Clone, Debug)]
pub struct RpcStatus {
    code: Code,
    message: String,
}

/// Subset of gRPC status codes used by the gateway.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Code {
    Ok = 0,
    Cancelled = 1,
    Unknown = 2,
    InvalidArgument = 3,
    DeadlineExceeded = 4,
    NotFound = 5,
    AlreadyExists = 6,
    PermissionDenied = 7,
    ResourceExhausted = 8,
    FailedPrecondition = 9,
    Aborted = 10,
    OutOfRange = 11,
    Unimplemented = 12,
    Internal = 13,
    Unavailable = 14,
    DataLoss = 15,
    Unauthenticated = 16,
}

impl Code {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Code::Ok,
            1 => Code::Cancelled,
            3 => Code::InvalidArgument,
            4 => Code::DeadlineExceeded,
            5 => Code::NotFound,
            12 => Code::Unimplemented,
            13 => Code::Internal,
            14 => Code::Unavailable,
            _ => Code::Unknown,
        }
    }
}

impl RpcStatus {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn ok() -> Self {
        Self::new(Code::Ok, "")
    }

    pub fn code(&self) -> Code {
        self.code
    }

    pub fn code_u32(&self) -> u32 {
        self.code as u32
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::new(Code::InvalidArgument, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(Code::Internal, msg)
    }

    pub fn deadline_exceeded(msg: impl Into<String>) -> Self {
        Self::new(Code::DeadlineExceeded, msg)
    }

    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self::new(Code::Unavailable, msg)
    }

    pub fn cancelled(msg: impl Into<String>) -> Self {
        Self::new(Code::Cancelled, msg)
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(Code::NotFound, msg)
    }

    pub fn unimplemented(msg: impl Into<String>) -> Self {
        Self::new(Code::Unimplemented, msg)
    }
}

impl fmt::Display for RpcStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rpc status {:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcStatus {}
