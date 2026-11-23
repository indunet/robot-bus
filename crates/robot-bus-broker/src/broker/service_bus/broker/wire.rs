//! Service-bus wire parse/build helpers.

/// Worker control commands (UTF-8 bytes, never protobuf).
pub(crate) const CMD_READY: &[u8] = b"READY";
pub(crate) const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
pub(crate) const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

/// Error body prefix written when no worker is registered for a service.
/// Wire convention: `b"NO_WORKER"` + `b'\0'` + service_name. End-side parses.
pub const ERR_NO_WORKER: &[u8] = b"NO_WORKER";

/// Error body prefix when an in-flight worker dies / disconnects before replying.
pub const ERR_WORKER_DIED: &[u8] = b"WORKER_DIED";

/// Cap poll timeout so the shutdown flag and pending-retry are responsive.
pub(crate) const POLL_CAP_MS: i64 = 200;

// ── Pure frame helpers (no sockets, unit-testable) ───────────────────────

/// Extract the service_name from client→broker frames `[client_id][svc][req_id][body]`.
pub fn parse_service_name(frames: &[Vec<u8>]) -> Option<&[u8]> {
    let svc = frames.get(1)?;
    if svc.is_empty() { None } else { Some(svc) }
}

/// Build the 5-frame message the broker sends to a worker via the backend
/// ROUTER: `[worker_id][client_id][svc][req_id][body]`.
pub fn build_worker_forward(
    worker_id: &[u8],
    client_id: &[u8],
    svc: &[u8],
    req_id: &[u8],
    body: &[u8],
) -> Vec<Vec<u8>> {
    vec![
        worker_id.to_vec(),
        client_id.to_vec(),
        svc.to_vec(),
        req_id.to_vec(),
        body.to_vec(),
    ]
}

/// Build the reply the broker sends to a client via the frontend ROUTER.
/// When `has_req_delim` is true (client used REQ), an empty delimiter frame
/// is inserted after the identity so REQ receives `[svc][req_id][body]`.
pub fn build_client_reply(
    client_id: &[u8],
    svc: &[u8],
    req_id: &[u8],
    body: &[u8],
    has_req_delim: bool,
) -> Vec<Vec<u8>> {
    if has_req_delim {
        vec![
            client_id.to_vec(),
            Vec::new(),
            svc.to_vec(),
            req_id.to_vec(),
            body.to_vec(),
        ]
    } else {
        vec![
            client_id.to_vec(),
            svc.to_vec(),
            req_id.to_vec(),
            body.to_vec(),
        ]
    }
}

/// Build an error body: `prefix` + `\0` + `service`.
pub fn build_error_body(prefix: &[u8], service: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(prefix.len() + 1 + service.len());
    v.extend_from_slice(prefix);
    v.push(0);
    v.extend_from_slice(service);
    v
}
