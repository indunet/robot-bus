//! Wire-level command / kind / error constants for action federation.

pub(super) const CMD_READY: &[u8] = b"READY";
pub(super) const CMD_READY_FED: &[u8] = b"READY_FED";
pub(super) const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
pub(super) const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

pub(super) const KIND_GOAL: &[u8] = b"GOAL";
pub(super) const KIND_FEEDBACK: &[u8] = b"FEEDBACK";
pub(super) const KIND_RESULT: &[u8] = b"RESULT";
pub(super) const KIND_CANCEL: &[u8] = b"CANCEL";

pub(super) const ERR_NO_WORKER: &[u8] = b"NO_WORKER";
pub(super) const ERR_WORKER_DIED: &[u8] = b"WORKER_DIED";
pub(super) const ERR_NO_GOAL: &[u8] = b"NO_GOAL";
pub(super) const ERR_CANCELLED: &[u8] = b"CANCELLED";

pub(super) const FED_ID_PREFIX: &str = "fed/";
pub(super) const POLL_CAP_MS: i64 = 200;
pub(super) const MAX_PENDING: usize = 64;
