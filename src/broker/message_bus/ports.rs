//! Port defaults aligned with `dji_app` `PhoneStreamTopics`.

pub const XSUB_PORT: u16 = 15560;
pub const XPUB_PORT: u16 = 15561;

pub const XSUB_CHANNEL: &str = "message_bus/xsub";
pub const XPUB_CHANNEL: &str = "message_bus/xpub";

pub const DEFAULT_XSUB_BIND: &str = "tcp://0.0.0.0:15560";
pub const DEFAULT_XPUB_BIND: &str = "tcp://0.0.0.0:15561";

/// Shallow ZMQ queues: prefer dropping over buffering for real-time streams.
pub const DEFAULT_SND_HWM: i32 = 2;
pub const DEFAULT_RCV_HWM: i32 = 2;
