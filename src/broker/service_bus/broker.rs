//! Service bus broker: dual-ROUTER load-balancing broker with worker registry.
//!
//! The broker parses only the UTF-8 `service_name` frame and the READY/HEARTBEAT/
//! DISCONNECT control frames. The request/response `body` frame is forwarded
//! as opaque bytes — no protobuf dependency.

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use zmq::Socket;

use super::ServiceBusConfig;

/// Worker control commands (UTF-8 bytes, never protobuf).
const CMD_READY: &[u8] = b"READY";
const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

/// Error body prefix written when no worker is registered for a service.
/// Wire convention: `b"NO_WORKER"` + `b'\0'` + service_name. End-side parses.
const ERR_NO_WORKER: &[u8] = b"NO_WORKER";

/// Cap poll timeout so the shutdown flag and pending-retry are responsive.
const POLL_CAP_MS: i64 = 200;

/// Max queued requests before the broker starts rejecting with NO_WORKER.
const MAX_PENDING: usize = 64;

#[derive(Clone, Debug)]
struct WorkerInfo {
    identity: Vec<u8>,
    last_heartbeat: Instant,
    in_flight: usize,
}

pub struct WorkerRegistry {
    /// service_name -> workers (round-robin load-balanced)
    workers: HashMap<String, Vec<WorkerInfo>>,
    /// worker identity -> service_name (reverse index for heartbeat/remove)
    by_identity: HashMap<Vec<u8>, String>,
    /// service_name -> next round-robin index
    rr_cursor: HashMap<String, usize>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            by_identity: HashMap::new(),
            rr_cursor: HashMap::new(),
        }
    }

    /// Register a worker for a service. Re-registration moves the worker to
    /// the new service (old binding is dropped first).
    pub fn register(&mut self, identity: Vec<u8>, service: String, now: Instant) {
        self.remove(&identity);
        self.by_identity.insert(identity.clone(), service.clone());
        self.workers.entry(service).or_default().push(WorkerInfo {
            identity,
            last_heartbeat: now,
            in_flight: 0,
        });
    }

    /// Refresh a worker's heartbeat timestamp.
    pub fn heartbeat(&mut self, identity: &[u8], now: Instant) {
        if let Some(svc) = self.by_identity.get(identity).cloned() {
            if let Some(list) = self.workers.get_mut(&svc) {
                if let Some(w) = list.iter_mut().find(|w| w.identity == identity) {
                    w.last_heartbeat = now;
                }
            }
        }
    }

    /// Remove a worker from the registry entirely.
    pub fn remove(&mut self, identity: &[u8]) {
        if let Some(svc) = self.by_identity.remove(identity) {
            if let Some(list) = self.workers.get_mut(&svc) {
                list.retain(|w| &w.identity != identity);
                if list.is_empty() {
                    self.workers.remove(&svc);
                    self.rr_cursor.remove(&svc);
                }
            }
        }
    }

    /// Evict workers whose last heartbeat is older than `timeout`.
    pub fn sweep_dead(&mut self, now: Instant, timeout: Duration) {
        let dead: Vec<Vec<u8>> = self
            .workers
            .values_mut()
            .flat_map(|list| {
                list.iter()
                    .filter(|w| now.duration_since(w.last_heartbeat) > timeout)
                    .map(|w| w.identity.clone())
                    .collect::<Vec<_>>()
            })
            .collect();
        for identity in dead {
            self.remove(&identity);
        }
    }

    /// Pick the next worker for a service (round-robin) and bump its in-flight count.
    pub fn select_worker(&mut self, service: &str) -> Option<Vec<u8>> {
        let list = self.workers.get_mut(service)?;
        if list.is_empty() {
            return None;
        }
        let cursor = self.rr_cursor.entry(service.to_string()).or_insert(0);
        let idx = *cursor % list.len();
        *cursor = (*cursor + 1) % list.len();
        list[idx].in_flight += 1;
        Some(list[idx].identity.clone())
    }

    /// Decrement a worker's in-flight count (called when its response arrives).
    pub fn release_worker(&mut self, identity: &[u8]) {
        if let Some(svc) = self.by_identity.get(identity).cloned() {
            if let Some(list) = self.workers.get_mut(&svc) {
                if let Some(w) = list.iter_mut().find(|w| &w.identity == identity) {
                    if w.in_flight > 0 {
                        w.in_flight -= 1;
                    }
                }
            }
        }
    }

    /// Number of workers registered for a service.
    pub fn worker_count(&self, service: &str) -> usize {
        self.workers.get(service).map(Vec::len).unwrap_or(0)
    }

    /// Whether a worker identity is currently registered.
    pub fn is_alive(&self, identity: &[u8]) -> bool {
        self.by_identity.contains_key(identity)
    }
}

/// A request waiting for an available worker.
struct PendingRequest {
    client_identity: Vec<u8>,
    service: Vec<u8>,
    request_id: Vec<u8>,
    body: Vec<u8>,
    has_req_delim: bool,
    queued_at: Instant,
}

// ── Pure frame helpers (no sockets, unit-testable) ───────────────────────

/// Extract the service_name from client→broker frames `[client_id][svc][req_id][body]`.
pub fn parse_service_name(frames: &[Vec<u8>]) -> Option<&[u8]> {
    let svc = frames.get(1)?;
    if svc.is_empty() {
        None
    } else {
        Some(svc)
    }
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

/// Build the error body written when no worker is available for a service.
pub fn build_error_body(service: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(ERR_NO_WORKER.len() + 1 + service.len());
    v.extend_from_slice(ERR_NO_WORKER);
    v.push(0);
    v.extend_from_slice(service);
    v
}

// ── Broker main loop ──────────────────────────────────────────────────────

/// Run the broker poll loop until `shutdown` is set.
pub fn run_loop(
    frontend: &Socket,
    backend: &Socket,
    config: &ServiceBusConfig,
    shutdown: &AtomicBool,
) -> Result<()> {
    let mut registry = WorkerRegistry::new();
    let mut pending: VecDeque<PendingRequest> = VecDeque::new();
    let mut client_is_req: HashMap<Vec<u8>, bool> = HashMap::new();
    let mut next_sweep =
        Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        let sweep_in = next_sweep.saturating_duration_since(Instant::now());
        let timeout_ms = (sweep_in.as_millis() as i64).min(POLL_CAP_MS).max(0);
        let mut items = [frontend.as_poll_item(zmq::POLLIN), backend.as_poll_item(zmq::POLLIN)];
        zmq::poll(&mut items, timeout_ms).context("poll")?;

        if items[0].get_revents().contains(zmq::POLLIN) {
            handle_client_request(frontend, backend, &mut registry, &mut pending, &mut client_is_req)?;
        }
        if items[1].get_revents().contains(zmq::POLLIN) {
            handle_worker_message(backend, frontend, &mut registry, &client_is_req)?;
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            registry.sweep_dead(now, Duration::from_millis(config.heartbeat_timeout_ms));
            retry_pending(frontend, backend, &mut pending, &mut registry, now)?;
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

fn handle_client_request(
    frontend: &Socket,
    backend: &Socket,
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    client_is_req: &mut HashMap<Vec<u8>, bool>,
) -> Result<()> {
    let frames = match frontend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("frontend recv_multipart"),
    };
    // REQ prepends an empty delimiter; ROUTER prepends client_id.
    // So frames: [client_id][empty][svc][req_id][body] (5 frames).
    // Normalize to [client_id][svc][req_id][body] by skipping the delimiter.
    if frames.len() < 4 {
        return Ok(()); // malformed, drop
    }
    let has_delim = frames.len() == 5 && frames[1].is_empty();
    let off = if has_delim { 2 } else { 1 };
    let client_id = &frames[0];
    let svc = &frames[off];
    let req_id = &frames[off + 1];
    let body = &frames[off + 2];

    if svc.is_empty() {
        return Ok(()); // empty service name, drop
    }
    let svc_str = match std::str::from_utf8(svc) {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(()),
    };
    client_is_req.insert(client_id.to_vec(), has_delim);

    if let Some(worker_id) = registry.select_worker(&svc_str) {
        let fwd = build_worker_forward(worker_id.as_slice(), client_id, svc, req_id, body);
        backend.send_multipart(fwd, 0).context("backend send forward")?;
    } else if pending.len() < MAX_PENDING {
        pending.push_back(PendingRequest {
            client_identity: client_id.to_vec(),
            service: svc.to_vec(),
            request_id: req_id.to_vec(),
            body: body.to_vec(),
            has_req_delim: has_delim,
            queued_at: Instant::now(),
        });
    } else {
        let err = build_error_body(svc);
        let reply = build_client_reply(client_id, svc, req_id, &err, has_delim);
        frontend.send_multipart(reply, 0).context("frontend send reject")?;
    }
    Ok(())
}

fn handle_worker_message(
    backend: &Socket,
    frontend: &Socket,
    registry: &mut WorkerRegistry,
    client_is_req: &HashMap<Vec<u8>, bool>,
) -> Result<()> {
    let frames = match backend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("backend recv_multipart"),
    };
    let worker_id = frames.first().map(|v| v.as_slice()).unwrap_or(&[]);

    match frames.len() {
        // control: [worker_id][cmd][svc]
        3 => {
            let cmd = &frames[1];
            let svc = &frames[2];
            if cmd == CMD_READY {
                let svc_str = String::from_utf8_lossy(svc).into_owned();
                registry.register(worker_id.to_vec(), svc_str, Instant::now());
            } else if cmd == CMD_HEARTBEAT {
                registry.heartbeat(worker_id, Instant::now());
            } else if cmd == CMD_DISCONNECT {
                registry.remove(worker_id);
            }
            Ok(())
        }
        // response: [worker_id][client_id][svc][req_id][body]
        5 => {
            registry.release_worker(worker_id);
            let client_id = &frames[1];
            let svc = &frames[2];
            let req_id = &frames[3];
            let body = &frames[4];
            let has_delim = client_is_req.get(client_id).copied().unwrap_or(false);
            let reply = build_client_reply(client_id, svc, req_id, body, has_delim);
            frontend.send_multipart(reply, 0).context("frontend send reply")?;
            Ok(())
        }
        _ => Ok(()), // unknown shape, drop
    }
}

/// Retry queued requests; give up (send NO_WORKER) for those stale beyond timeout.
fn retry_pending(
    frontend: &Socket,
    backend: &Socket,
    pending: &mut VecDeque<PendingRequest>,
    registry: &mut WorkerRegistry,
    now: Instant,
) -> Result<()> {
    let mut still_pending = VecDeque::new();
    let timeout = Duration::from_secs(5);
    while let Some(req) = pending.pop_front() {
        let svc_str = match std::str::from_utf8(&req.service) {
            Ok(s) => s.to_string(),
            Err(_) => continue, // drop malformed
        };
        if let Some(worker_id) = registry.select_worker(&svc_str) {
            let fwd = build_worker_forward(
                worker_id.as_slice(),
                &req.client_identity,
                &req.service,
                &req.request_id,
                &req.body,
            );
            backend.send_multipart(fwd, 0).context("backend send pending forward")?;
        } else if now.duration_since(req.queued_at) > timeout {
            let err = build_error_body(&req.service);
            let reply = build_client_reply(
                &req.client_identity,
                &req.service,
                &req.request_id,
                &err,
                req.has_req_delim,
            );
            frontend
                .send_multipart(reply, 0)
                .context("frontend send pending reject")?;
        } else {
            still_pending.push_back(req);
        }
    }
    *pending = still_pending;
    Ok(())
}

// ── Unit tests (no sockets, no ports) ─────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Instant {
        Instant::now()
    }

    #[test]
    fn registry_register_and_select() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc.x".into(), t);
        assert_eq!(r.worker_count("svc.x"), 1);
        assert!(r.is_alive(b"w1"));
        let picked = r.select_worker("svc.x");
        assert_eq!(picked, Some(b"w1".to_vec()));
    }

    #[test]
    fn registry_select_none_when_empty() {
        let mut r = WorkerRegistry::new();
        assert_eq!(r.select_worker("svc.missing"), None);
        assert_eq!(r.worker_count("svc.missing"), 0);
    }

    #[test]
    fn registry_round_robin_balances() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc".into(), t);
        r.register(b"w2".to_vec(), "svc".into(), t);
        // round-robin: first -> w1, second -> w2, third -> w1
        let first = r.select_worker("svc").unwrap();
        let second = r.select_worker("svc").unwrap();
        let third = r.select_worker("svc").unwrap();
        assert_eq!(first, b"w1".to_vec());
        assert_eq!(second, b"w2".to_vec());
        assert_eq!(third, b"w1".to_vec());
    }

    #[test]
    fn registry_release_decrements() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc".into(), t);
        let _ = r.select_worker("svc");
        r.release_worker(b"w1");
        // after release, in_flight back to 0; select still returns w1
        assert_eq!(r.select_worker("svc"), Some(b"w1".to_vec()));
    }

    #[test]
    fn registry_heartbeat_refreshes() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc".into(), t);
        r.heartbeat(b"w1", t + Duration::from_secs(1));
        assert!(r.is_alive(b"w1"));
    }

    #[test]
    fn registry_sweep_evicts_dead() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc".into(), t);
        // simulate heartbeat far in the past via a large timeout
        r.sweep_dead(t + Duration::from_secs(10), Duration::from_secs(1));
        assert!(!r.is_alive(b"w1"));
        assert_eq!(r.worker_count("svc"), 0);
    }

    #[test]
    fn registry_remove() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc".into(), t);
        r.remove(b"w1");
        assert!(!r.is_alive(b"w1"));
        assert_eq!(r.worker_count("svc"), 0);
    }

    #[test]
    fn registry_reregister_moves_service() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "svc.a".into(), t);
        r.register(b"w1".to_vec(), "svc.b".into(), t);
        assert_eq!(r.worker_count("svc.a"), 0);
        assert_eq!(r.worker_count("svc.b"), 1);
        assert_eq!(r.select_worker("svc.b"), Some(b"w1".to_vec()));
    }

    #[test]
    fn parse_service_name_valid() {
        let frames = vec![
            b"client1".to_vec(),
            b"svc.x".to_vec(),
            b"req1".to_vec(),
            b"body".to_vec(),
        ];
        assert_eq!(parse_service_name(&frames), Some(b"svc.x".as_slice()));
    }

    #[test]
    fn parse_service_name_missing() {
        assert_eq!(parse_service_name(&[b"only".to_vec()]), None);
        assert_eq!(parse_service_name(&[]), None);
    }

    #[test]
    fn parse_service_name_empty() {
        let frames = vec![b"client".to_vec(), b"".to_vec(), b"req".to_vec()];
        assert_eq!(parse_service_name(&frames), None);
    }

    #[test]
    fn build_worker_forward_order() {
        let fwd = build_worker_forward(b"wid", b"cid", b"svc", b"rid", b"body");
        assert_eq!(
            fwd,
            vec![
                b"wid".to_vec(),
                b"cid".to_vec(),
                b"svc".to_vec(),
                b"rid".to_vec(),
                b"body".to_vec(),
            ]
        );
    }

    #[test]
    fn build_client_reply_order_req() {
        let reply = build_client_reply(b"cid", b"svc", b"rid", b"body", true);
        assert_eq!(
            reply,
            vec![
                b"cid".to_vec(),
                b"".to_vec(),
                b"svc".to_vec(),
                b"rid".to_vec(),
                b"body".to_vec(),
            ]
        );
    }

    #[test]
    fn build_client_reply_order_dealer() {
        let reply = build_client_reply(b"cid", b"svc", b"rid", b"body", false);
        assert_eq!(
            reply,
            vec![b"cid".to_vec(), b"svc".to_vec(), b"rid".to_vec(), b"body".to_vec()]
        );
    }

    #[test]
    fn build_error_body_format() {
        let err = build_error_body(b"svc.x");
        assert_eq!(&err[..9], b"NO_WORKER");
        assert_eq!(err[9], 0);
        assert_eq!(&err[10..], b"svc.x");
    }
}
