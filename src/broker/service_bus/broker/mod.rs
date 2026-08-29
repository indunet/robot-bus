//! Service bus broker: dual-ROUTER load-balancing broker with worker registry.
//!
//! The broker parses only the UTF-8 `service_name` frame and the READY/HEARTBEAT/
//! DISCONNECT control frames. The request/response `body` frame is forwarded
//! as opaque bytes — no protobuf dependency.

mod inflight;
mod registry;
mod wire;

pub use inflight::{InFlightEntry, InFlightTable};
pub(crate) use inflight::{extend_hops, hop_contains};
pub use registry::{WorkerRegistry, WorkerSource};
pub use wire::{
    ERR_NO_WORKER, ERR_WORKER_DIED, build_client_reply, build_error_body, build_worker_forward,
    parse_service_name,
};

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use zmq::Socket;

use super::ServiceBusConfig;
use super::metrics::ServiceMetrics;
use inflight::PendingRequest;
use wire::{CMD_DISCONNECT, CMD_HEARTBEAT, CMD_READY, POLL_CAP_MS};

// ── Broker main loop ──────────────────────────────────────────────────────

/// Run the broker poll loop until `shutdown` is set.
pub fn run_loop(
    frontend: &Socket,
    backend: &Socket,
    config: &ServiceBusConfig,
    shutdown: &AtomicBool,
    metrics: Option<&Arc<ServiceMetrics>>,
) -> Result<()> {
    let mut registry = WorkerRegistry::new();
    let mut pending: VecDeque<PendingRequest> = VecDeque::new();
    let mut in_flight = InFlightTable::new();
    let mut client_is_req: HashMap<Vec<u8>, bool> = HashMap::new();
    let mut next_sweep = Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);
    let pending_timeout = Duration::from_millis(config.pending_timeout_ms);
    let max_pending = config.max_pending;

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        let sweep_in = next_sweep.saturating_duration_since(Instant::now());
        let timeout_ms = (sweep_in.as_millis() as i64).min(POLL_CAP_MS).max(0);
        let mut items = [
            frontend.as_poll_item(zmq::POLLIN),
            backend.as_poll_item(zmq::POLLIN),
        ];
        zmq::poll(&mut items, timeout_ms).context("poll")?;

        if items[0].get_revents().contains(zmq::POLLIN) {
            handle_client_request(
                frontend,
                backend,
                &mut registry,
                &mut pending,
                &mut in_flight,
                &mut client_is_req,
                max_pending,
                metrics,
            )?;
        }
        if items[1].get_revents().contains(zmq::POLLIN) {
            handle_worker_message(
                backend,
                frontend,
                &mut registry,
                &mut in_flight,
                &client_is_req,
                metrics,
            )?;
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            let dead = registry.sweep_dead(now, Duration::from_millis(config.heartbeat_timeout_ms));
            for wid in dead {
                reclaim_worker_requests(frontend, &mut in_flight, &wid, metrics)?;
            }
            retry_pending(
                frontend,
                backend,
                &mut pending,
                &mut registry,
                &mut in_flight,
                now,
                pending_timeout,
                metrics,
            )?;
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

/// Send synthetic WORKER_DIED replies for in-flight requests owned by `worker_identity`.
pub fn reclaim_worker_requests(
    frontend: &Socket,
    in_flight: &mut InFlightTable,
    worker_identity: &[u8],
    metrics: Option<&Arc<ServiceMetrics>>,
) -> Result<()> {
    let dropped = in_flight.evict_worker(worker_identity);
    for e in dropped {
        if let Some(m) = metrics {
            let name = String::from_utf8_lossy(&e.service);
            m.record_error(&name);
            m.record_worker_died(&name);
        }
        let err = build_error_body(ERR_WORKER_DIED, &e.service);
        let reply = build_client_reply(
            &e.client_identity,
            &e.service,
            &e.request_id,
            &err,
            e.has_req_delim,
        );
        frontend
            .send_multipart(reply, 0)
            .context("frontend send worker-died reply")?;
    }
    Ok(())
}

fn handle_client_request(
    frontend: &Socket,
    backend: &Socket,
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    in_flight: &mut InFlightTable,
    client_is_req: &mut HashMap<Vec<u8>, bool>,
    max_pending: usize,
    metrics: Option<&Arc<ServiceMetrics>>,
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
        if let Some(m) = metrics {
            m.record_call_start(&svc_str, client_id, req_id);
        }
        in_flight.insert(InFlightEntry {
            client_identity: client_id.to_vec(),
            worker_identity: worker_id.clone(),
            service: svc.to_vec(),
            request_id: req_id.to_vec(),
            has_req_delim: has_delim,
        });
        let fwd = build_worker_forward(worker_id.as_slice(), client_id, svc, req_id, body);
        backend
            .send_multipart(fwd, 0)
            .context("backend send forward")?;
    } else if pending.len() < max_pending {
        pending.push_back(PendingRequest {
            client_identity: client_id.to_vec(),
            service: svc.to_vec(),
            request_id: req_id.to_vec(),
            body: body.to_vec(),
            has_req_delim: has_delim,
            queued_at: Instant::now(),
        });
    } else {
        if let Some(m) = metrics {
            m.record_error(&svc_str);
        }
        let err = build_error_body(ERR_NO_WORKER, svc);
        let reply = build_client_reply(client_id, svc, req_id, &err, has_delim);
        frontend
            .send_multipart(reply, 0)
            .context("frontend send reject")?;
    }
    Ok(())
}

fn handle_worker_message(
    backend: &Socket,
    frontend: &Socket,
    registry: &mut WorkerRegistry,
    in_flight: &mut InFlightTable,
    client_is_req: &HashMap<Vec<u8>, bool>,
    metrics: Option<&Arc<ServiceMetrics>>,
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
                registry.register(worker_id.to_vec(), svc_str.clone(), Instant::now());
                if let Some(m) = metrics {
                    m.set_workers(&svc_str, registry.worker_count(&svc_str) as u64);
                }
            } else if cmd == CMD_HEARTBEAT {
                registry.heartbeat(worker_id, Instant::now());
            } else if cmd == CMD_DISCONNECT {
                let svcs = registry.services_of(worker_id);
                registry.remove(worker_id);
                reclaim_worker_requests(frontend, in_flight, worker_id, metrics)?;
                if let Some(m) = metrics {
                    for s in svcs {
                        m.set_workers(&s, registry.worker_count(&s) as u64);
                    }
                }
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
            let tracked = in_flight.remove(client_id, req_id);
            if let Some(m) = metrics {
                if let Ok(svc_str) = std::str::from_utf8(svc) {
                    m.record_call_ok(svc_str, client_id, req_id);
                }
            }
            let has_delim = tracked
                .map(|e| e.has_req_delim)
                .unwrap_or_else(|| client_is_req.get(client_id).copied().unwrap_or(false));
            let reply = build_client_reply(client_id, svc, req_id, body, has_delim);
            frontend
                .send_multipart(reply, 0)
                .context("frontend send reply")?;
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
    in_flight: &mut InFlightTable,
    now: Instant,
    pending_timeout: Duration,
    metrics: Option<&Arc<ServiceMetrics>>,
) -> Result<()> {
    let mut still_pending = VecDeque::new();
    while let Some(req) = pending.pop_front() {
        let svc_str = match std::str::from_utf8(&req.service) {
            Ok(s) => s.to_string(),
            Err(_) => continue, // drop malformed
        };
        if let Some(worker_id) = registry.select_worker(&svc_str) {
            if let Some(m) = metrics {
                m.record_call_start(&svc_str, &req.client_identity, &req.request_id);
            }
            in_flight.insert(InFlightEntry {
                client_identity: req.client_identity.clone(),
                worker_identity: worker_id.clone(),
                service: req.service.clone(),
                request_id: req.request_id.clone(),
                has_req_delim: req.has_req_delim,
            });
            let fwd = build_worker_forward(
                worker_id.as_slice(),
                &req.client_identity,
                &req.service,
                &req.request_id,
                &req.body,
            );
            backend
                .send_multipart(fwd, 0)
                .context("backend send pending forward")?;
        } else if now.duration_since(req.queued_at) > pending_timeout {
            if let Some(m) = metrics {
                m.record_error(&svc_str);
                m.record_pending_timeout(&svc_str);
            }
            let err = build_error_body(ERR_NO_WORKER, &req.service);
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
            vec![
                b"cid".to_vec(),
                b"svc".to_vec(),
                b"rid".to_vec(),
                b"body".to_vec()
            ]
        );
    }

    #[test]
    fn build_error_body_format() {
        let err = build_error_body(ERR_NO_WORKER, b"svc.x");
        assert_eq!(&err[..9], b"NO_WORKER");
        assert_eq!(err[9], 0);
        assert_eq!(&err[10..], b"svc.x");
    }

    #[test]
    fn build_error_body_worker_died() {
        let err = build_error_body(ERR_WORKER_DIED, b"svc.x");
        assert_eq!(&err[..11], b"WORKER_DIED");
        assert_eq!(err[11], 0);
        assert_eq!(&err[12..], b"svc.x");
    }

    #[test]
    fn inflight_evict_worker() {
        let mut t = InFlightTable::new();
        t.insert(InFlightEntry {
            client_identity: b"c1".to_vec(),
            worker_identity: b"w1".to_vec(),
            service: b"svc".to_vec(),
            request_id: b"r1".to_vec(),
            has_req_delim: true,
        });
        t.insert(InFlightEntry {
            client_identity: b"c2".to_vec(),
            worker_identity: b"w2".to_vec(),
            service: b"svc".to_vec(),
            request_id: b"r2".to_vec(),
            has_req_delim: false,
        });
        let dropped = t.evict_worker(b"w1");
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].request_id, b"r1");
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn registry_local_preferred_over_federated() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register_federated(
            b"fed/b".to_vec(),
            "svc".into(),
            t,
            "b".into(),
            "b".into(),
            "b".into(),
        );
        r.register(b"local".to_vec(), "svc".into(), t);
        assert_eq!(r.select_worker("svc"), Some(b"local".to_vec()));
        assert_eq!(r.select_worker("svc"), Some(b"local".to_vec()));
    }

    #[test]
    fn registry_select_avoids_hops() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register_federated(
            b"fed/a".to_vec(),
            "svc".into(),
            t,
            "a".into(),
            "a".into(),
            "a".into(),
        );
        assert_eq!(r.select_worker_avoiding_hops("svc", "a"), None);
        assert_eq!(
            r.select_worker_avoiding_hops("svc", "c"),
            Some(b"fed/a".to_vec())
        );
    }

    #[test]
    fn registry_federated_multi_service_same_identity() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register_federated(
            b"fed/b".to_vec(),
            "svc.a".into(),
            t,
            "b".into(),
            "b".into(),
            "b".into(),
        );
        r.register_federated(
            b"fed/b".to_vec(),
            "svc.b".into(),
            t,
            "b".into(),
            "b".into(),
            "b".into(),
        );
        assert_eq!(r.worker_count("svc.a"), 1);
        assert_eq!(r.worker_count("svc.b"), 1);
        assert_eq!(r.services_of(b"fed/b").len(), 2);
    }

    #[test]
    fn hop_extend_and_contains() {
        assert!(!hop_contains("", "a"));
        assert!(hop_contains("a,b", "b"));
        assert_eq!(extend_hops("", "a"), "a");
        assert_eq!(extend_hops("a", "b"), "a,b");
    }
}
