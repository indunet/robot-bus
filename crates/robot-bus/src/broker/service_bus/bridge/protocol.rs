//! Wire-level command / identity constants for service federation.

pub(super) const CMD_READY: &[u8] = b"READY";
pub(super) const CMD_READY_FED: &[u8] = b"READY_FED";
pub(super) const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
pub(super) const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

/// One-sided join (see `remote::PeerLink`): sent by the joining broker, answered with
/// `JOIN_ACK` (`[id][epoch][DEFER]`).
pub(super) const CMD_JOIN: &[u8] = b"JOIN";
pub(super) const CMD_JOIN_ACK: &[u8] = b"JOIN_ACK";
/// Request from the joining broker to the joined one: `[FED_CALL][corr][svc][req_id][body]`.
pub(super) const CMD_FED_CALL: &[u8] = b"FED_CALL";
/// Reply of the joined broker: `[FED_REPLY][corr][svc][req_id][body]`.
pub(super) const CMD_FED_REPLY: &[u8] = b"FED_REPLY";
/// `JOIN_ACK` flag: both brokers configured each other; the answering broker owns the link.
pub(super) const DEFER: &[u8] = b"DEFER";

pub(super) const FED_ID_PREFIX: &str = "fed/";
/// Registry identity of services reachable through our own joining link (calls go out
/// over the DEALER as `FED_CALL`).
pub(super) const LINK_ID_PREFIX: &str = "fedlink/";
pub(super) const FED_REQ_PREFIX: &str = "fedreq/";

pub(super) const POLL_CAP_MS: i64 = 200;
