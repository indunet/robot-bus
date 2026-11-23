//! Service-bus bind defaults.
//!
//! TCP defaults use port `0` so the OS assigns a free port at bind time.

pub const FRONTEND_PORT: u16 = 0;
pub const BACKEND_PORT: u16 = 0;

pub const FRONTEND_CHANNEL: &str = "service_bus/frontend";
pub const BACKEND_CHANNEL: &str = "service_bus/backend";

pub const DEFAULT_FRONTEND_BIND: &str = "tcp://0.0.0.0:0";
pub const DEFAULT_BACKEND_BIND: &str = "tcp://0.0.0.0:0";

/// Request/response traffic is not a real-time stream; allow a slightly deeper
/// queue than `message_bus` (which uses 2 to prefer dropping over buffering).
pub const DEFAULT_SND_HWM: i32 = 4;
pub const DEFAULT_RCV_HWM: i32 = 4;

/// Worker heartbeat protocol.
///
/// Workers send `HEARTBEAT` every `HEARTBEAT_INTERVAL_MS`; the broker evicts a
/// worker if no frame is seen within `HEARTBEAT_TIMEOUT_MS`.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 2500;
pub const DEFAULT_HEARTBEAT_TIMEOUT_MS: u64 = 7500;

/// How long a request may sit queued waiting for a worker before `NO_WORKER`.
pub const DEFAULT_PENDING_TIMEOUT_MS: u64 = 5000;

/// Max queued requests before the broker starts rejecting with `NO_WORKER`.
pub const DEFAULT_MAX_PENDING: usize = 64;
