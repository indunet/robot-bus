//! Message-bus bind defaults.
//!
//! TCP defaults use port `0` so the OS assigns a free port at bind time.
//! Clients learn the resolved ports via `GET /api/v1/discover` (API listen).

pub const XSUB_PORT: u16 = 0;
pub const XPUB_PORT: u16 = 0;

pub const XSUB_CHANNEL: &str = "message_bus/xsub";
pub const XPUB_CHANNEL: &str = "message_bus/xpub";

pub const DEFAULT_XSUB_BIND: &str = "tcp://0.0.0.0:0";
pub const DEFAULT_XPUB_BIND: &str = "tcp://0.0.0.0:0";

/// Shallow ZMQ queues: prefer dropping over buffering for real-time streams.
pub const DEFAULT_SND_HWM: i32 = 8;
pub const DEFAULT_RCV_HWM: i32 = 8;
