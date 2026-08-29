//! Action-bus wire parse/build helpers.

/// Worker control commands (UTF-8 bytes, never protobuf).
pub(crate) const CMD_READY: &[u8] = b"READY";
pub(crate) const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
pub(crate) const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

/// Client→broker and worker→broker message-kind tokens (UTF-8 bytes).
pub(crate) const KIND_GOAL: &[u8] = b"GOAL";
pub(crate) const KIND_FEEDBACK: &[u8] = b"FEEDBACK";
pub(crate) const KIND_RESULT: &[u8] = b"RESULT";
pub(crate) const KIND_CANCEL: &[u8] = b"CANCEL";

/// Error body prefix written when no worker is registered for an action.
/// Wire convention: `b"NO_WORKER"` + `b'\0'` + action_name. End-side parses.
pub(crate) const ERR_NO_WORKER: &[u8] = b"NO_WORKER";

/// Error body prefix written when the worker owning an in-flight goal died.
/// Wire convention: `b"WORKER_DIED"` + `b'\0'` + action_name.
pub(crate) const ERR_WORKER_DIED: &[u8] = b"WORKER_DIED";

/// Error body prefix written when a CANCEL arrives for an unknown/finished goal.
pub(crate) const ERR_NO_GOAL: &[u8] = b"NO_GOAL";

/// Error body prefix when a pending (not yet dispatched) goal is cancelled.
pub(crate) const ERR_CANCELLED: &[u8] = b"CANCELLED";

/// Cap poll timeout so the shutdown flag and pending-retry are responsive.
pub(crate) const POLL_CAP_MS: i64 = 200;

/// Max queued goals before the broker starts rejecting with NO_WORKER.
pub(crate) const MAX_PENDING: usize = 64;

/// Soft cap on in-flight goals in the GoalTable.
pub(crate) const MAX_GOALS: usize = 1024;

// ── Pure frame helpers (no sockets, unit-testable) ───────────────────────

/// Parse the client→broker GOAL/CANCEL frames.
///
/// Client (DEALER) sends: `[action_name][goal_id][kind][body]` (4 frames).
/// DEALER does NOT insert an empty delimiter, so no stripping is needed.
/// Only GOAL and CANCEL originate from the client; FEEDBACK/RESULT flow
/// worker→broker, so they are rejected here.
pub fn parse_client_message(frames: &[Vec<u8>]) -> Option<ClientMessage<'_>> {
    if frames.len() != 4 {
        return None;
    }
    let action = frames[0].as_slice();
    let goal_id = frames[1].as_slice();
    let kind = frames[2].as_slice();
    let body = frames[3].as_slice();
    if action.is_empty() || goal_id.is_empty() {
        return None;
    }
    let kind_enum = if kind == KIND_GOAL {
        ClientKind::Goal
    } else if kind == KIND_CANCEL {
        ClientKind::Cancel
    } else {
        return None;
    };
    Some(ClientMessage {
        action,
        goal_id,
        kind: kind_enum,
        body,
    })
}

pub struct ClientMessage<'a> {
    pub action: &'a [u8],
    pub goal_id: &'a [u8],
    pub kind: ClientKind,
    pub body: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientKind {
    Goal,
    Cancel,
}

/// Parse the worker→broker frames. Returns `None` for malformed shapes.
///
/// Control (broker sees 3 frames): `[worker_id][cmd][action_name]`.
/// Feedback/result (broker sees 6 frames):
/// `[worker_id][client_id][action_name][goal_id][kind][body]`.
pub fn parse_worker_message(frames: &[Vec<u8>]) -> Option<WorkerMessage<'_>> {
    match frames.len() {
        3 => {
            let worker_id = frames[0].as_slice();
            let cmd = frames[1].as_slice();
            let action = frames[2].as_slice();
            let ctrl = if cmd == CMD_READY {
                WorkerControl::Ready
            } else if cmd == CMD_HEARTBEAT {
                WorkerControl::Heartbeat
            } else if cmd == CMD_DISCONNECT {
                WorkerControl::Disconnect
            } else {
                return None;
            };
            Some(WorkerMessage::Control {
                worker_id,
                action,
                control: ctrl,
            })
        }
        6 => {
            let worker_id = frames[0].as_slice();
            let client_id = frames[1].as_slice();
            let action = frames[2].as_slice();
            let goal_id = frames[3].as_slice();
            let kind = frames[4].as_slice();
            let body = frames[5].as_slice();
            let k = if kind == KIND_FEEDBACK {
                WorkerKind::Feedback
            } else if kind == KIND_RESULT {
                WorkerKind::Result
            } else {
                return None;
            };
            Some(WorkerMessage::Response {
                worker_id,
                client_id,
                action,
                goal_id,
                kind: k,
                body,
            })
        }
        _ => None,
    }
}

pub enum WorkerMessage<'a> {
    Control {
        worker_id: &'a [u8],
        action: &'a [u8],
        control: WorkerControl,
    },
    Response {
        worker_id: &'a [u8],
        client_id: &'a [u8],
        action: &'a [u8],
        goal_id: &'a [u8],
        kind: WorkerKind,
        body: &'a [u8],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerControl {
    Ready,
    Heartbeat,
    Disconnect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerKind {
    Feedback,
    Result,
}

/// Build the 6-frame message the broker sends to a worker via the backend
/// ROUTER for a GOAL: `[worker_id][client_id][action][goal_id][GOAL][body]`.
pub fn build_worker_goal(
    worker_id: &[u8],
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    body: &[u8],
) -> Vec<Vec<u8>> {
    vec![
        worker_id.to_vec(),
        client_id.to_vec(),
        action.to_vec(),
        goal_id.to_vec(),
        KIND_GOAL.to_vec(),
        body.to_vec(),
    ]
}

/// Build the 6-frame message the broker sends to a worker for a CANCEL:
/// `[worker_id][client_id][action][goal_id][CANCEL][body]`.
pub fn build_worker_cancel(
    worker_id: &[u8],
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    body: &[u8],
) -> Vec<Vec<u8>> {
    vec![
        worker_id.to_vec(),
        client_id.to_vec(),
        action.to_vec(),
        goal_id.to_vec(),
        KIND_CANCEL.to_vec(),
        body.to_vec(),
    ]
}

/// Build the reply the broker sends to a client (DEALER) via the frontend
/// ROUTER. DEALER expects no empty delimiter:
/// `[client_id][action][goal_id][kind][body]`.
pub fn build_client_reply(
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    kind: &[u8],
    body: &[u8],
) -> Vec<Vec<u8>> {
    vec![
        client_id.to_vec(),
        action.to_vec(),
        goal_id.to_vec(),
        kind.to_vec(),
        body.to_vec(),
    ]
}

/// Build the error body: `prefix` + `0x00` + `action`.
pub fn build_error_body(prefix: &[u8], action: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(prefix.len() + 1 + action.len());
    v.extend_from_slice(prefix);
    v.push(0);
    v.extend_from_slice(action);
    v
}

pub(crate) fn kind_as_bytes(kind: WorkerKind) -> &'static [u8] {
    match kind {
        WorkerKind::Feedback => KIND_FEEDBACK,
        WorkerKind::Result => KIND_RESULT,
    }
}
