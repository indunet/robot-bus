//! Wire-level command / identity constants for service federation.

pub(super) const CMD_READY: &[u8] = b"READY";
pub(super) const CMD_READY_FED: &[u8] = b"READY_FED";
pub(super) const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
pub(super) const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

pub(super) const FED_ID_PREFIX: &str = "fed/";
pub(super) const FED_REQ_PREFIX: &str = "fedreq/";

pub(super) const POLL_CAP_MS: i64 = 200;
