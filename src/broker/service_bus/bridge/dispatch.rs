//! Request dispatch, reply delivery, and pending / corr-id bookkeeping.

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use zmq::Socket;

use super::remote::{
    is_fed_identity, make_corr_id, CorrEntry, PeerLink, PendingRequest, ReplyTarget,
};
use super::super::broker::{
    build_client_reply, build_error_body, build_worker_forward, reclaim_worker_requests,
    InFlightEntry, InFlightTable, WorkerRegistry, ERR_NO_WORKER, ERR_WORKER_DIED,
};

fn clone_reply_target(t: &ReplyTarget) -> ReplyTarget {
    match t {
        ReplyTarget::Frontend { has_req_delim } => ReplyTarget::Frontend {
            has_req_delim: *has_req_delim,
        },
        ReplyTarget::Peer { peer_idx } => ReplyTarget::Peer {
            peer_idx: *peer_idx,
        },
    }
}

fn send_error_to_target(
    frontend: &Socket,
    peers: Option<&[PeerLink]>,
    target: &ReplyTarget,
    original_client_id: &[u8],
    svc: &[u8],
    req_id: &[u8],
    err: &[u8],
) -> Result<()> {
    match target {
        ReplyTarget::Frontend { has_req_delim } => {
            let reply =
                build_client_reply(original_client_id, svc, req_id, err, *has_req_delim);
            frontend
                .send_multipart(reply, 0)
                .context("frontend send error")?;
        }
        ReplyTarget::Peer { peer_idx } => {
            if let Some(peers) = peers {
                let _ = peers[*peer_idx].dealer.send_multipart(
                    [original_client_id, svc, req_id, err],
                    zmq::DONTWAIT,
                );
            }
        }
    }
    Ok(())
}

pub(super) fn dispatch_request(
    frontend: &Socket,
    backend: &Socket,
    peers: Option<&mut [PeerLink]>,
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    in_flight: &mut InFlightTable,
    broker_id: &str,
    client_id: &[u8],
    svc: &[u8],
    svc_str: &str,
    req_id: &[u8],
    body: &[u8],
    hop_path: &str,
    reply_target: ReplyTarget,
    peer_idx: Option<usize>,
    has_req_delim: bool,
    max_pending: usize,
    metrics: Option<&std::sync::Arc<crate::broker::service_bus::ServiceMetrics>>,
) -> Result<()> {
    let hops_for_select = if hop_path.is_empty() {
        broker_id.to_string()
    } else {
        hop_path.to_string()
    };

    if let Some(worker_id) = registry.select_worker_avoiding_hops(svc_str, &hops_for_select) {
        let need_corr =
            is_fed_identity(&worker_id) || matches!(reply_target, ReplyTarget::Peer { .. });
        let forward_client_id = if need_corr {
            let cid = make_corr_id(&hops_for_select);
            corr.insert(
                cid.clone(),
                CorrEntry {
                    target: clone_reply_target(&reply_target),
                    original_client_id: client_id.to_vec(),
                    service: svc.to_vec(),
                    request_id: req_id.to_vec(),
                    worker_identity: worker_id.clone(),
                    queued_at: Instant::now(),
                },
            );
            cid
        } else {
            in_flight.insert(InFlightEntry {
                client_identity: client_id.to_vec(),
                worker_identity: worker_id.clone(),
                service: svc.to_vec(),
                request_id: req_id.to_vec(),
                has_req_delim,
            });
            client_id.to_vec()
        };

        if let Some(m) = metrics {
            m.record_call_start(svc_str, client_id, req_id);
        }
        let fwd = build_worker_forward(
            worker_id.as_slice(),
            &forward_client_id,
            svc,
            req_id,
            body,
        );
        backend
            .send_multipart(fwd, 0)
            .context("backend send forward")?;
    } else if pending.len() < max_pending {
        pending.push_back(PendingRequest {
            client_identity: client_id.to_vec(),
            service: svc.to_vec(),
            request_id: req_id.to_vec(),
            body: body.to_vec(),
            has_req_delim,
            hop_path: hops_for_select,
            peer_idx,
            queued_at: Instant::now(),
        });
    } else {
        if let Some(m) = metrics {
            m.record_error(svc_str);
        }
        let err = build_error_body(ERR_NO_WORKER, svc);
        match (peer_idx, peers) {
            (Some(idx), Some(peers)) => {
                let _ = peers[idx].dealer.send_multipart(
                    [client_id, svc, req_id, err.as_slice()],
                    zmq::DONTWAIT,
                );
            }
            _ => {
                let reply = build_client_reply(client_id, svc, req_id, &err, has_req_delim);
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send reject")?;
            }
        }
    }
    Ok(())
}

pub(super) fn deliver_reply(
    frontend: &Socket,
    peers: &mut [PeerLink],
    client_is_req: &HashMap<Vec<u8>, bool>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    in_flight: &mut InFlightTable,
    client_id: &[u8],
    svc: &[u8],
    req_id: &[u8],
    body: &[u8],
    metrics: Option<&std::sync::Arc<crate::broker::service_bus::ServiceMetrics>>,
) -> Result<()> {
    if let Some(entry) = corr.remove(client_id) {
        match entry.target {
            ReplyTarget::Frontend { has_req_delim } => {
                if let Some(m) = metrics {
                    if let Ok(svc_str) = std::str::from_utf8(svc) {
                        m.record_call_ok(svc_str, &entry.original_client_id, req_id);
                    }
                }
                let reply = build_client_reply(
                    &entry.original_client_id,
                    svc,
                    req_id,
                    body,
                    has_req_delim,
                );
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send reply")?;
            }
            ReplyTarget::Peer { peer_idx } => {
                let _ = peers[peer_idx].dealer.send_multipart(
                    [
                        entry.original_client_id.as_slice(),
                        svc,
                        req_id,
                        body,
                    ],
                    zmq::DONTWAIT,
                );
            }
        }
        return Ok(());
    }

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

pub(super) fn retry_pending(
    frontend: &Socket,
    backend: &Socket,
    peers: &[PeerLink],
    pending: &mut VecDeque<PendingRequest>,
    registry: &mut WorkerRegistry,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    in_flight: &mut InFlightTable,
    broker_id: &str,
    now: Instant,
    pending_timeout: Duration,
    max_pending: usize,
    metrics: Option<&std::sync::Arc<crate::broker::service_bus::ServiceMetrics>>,
) -> Result<()> {
    let _ = max_pending;
    let mut still = VecDeque::new();
    while let Some(req) = pending.pop_front() {
        let svc_str = match std::str::from_utf8(&req.service) {
            Ok(s) => s.to_string(),
            Err(_) => continue,
        };
        let hops = if req.hop_path.is_empty() {
            broker_id.to_string()
        } else {
            req.hop_path.clone()
        };
        if let Some(worker_id) = registry.select_worker_avoiding_hops(&svc_str, &hops) {
            let reply_target = if let Some(idx) = req.peer_idx {
                ReplyTarget::Peer { peer_idx: idx }
            } else {
                ReplyTarget::Frontend {
                    has_req_delim: req.has_req_delim,
                }
            };
            let need_corr =
                is_fed_identity(&worker_id) || matches!(reply_target, ReplyTarget::Peer { .. });
            let forward_client_id = if need_corr {
                let cid = make_corr_id(&hops);
                corr.insert(
                    cid.clone(),
                    CorrEntry {
                        target: reply_target,
                        original_client_id: req.client_identity.clone(),
                        service: req.service.clone(),
                        request_id: req.request_id.clone(),
                        worker_identity: worker_id.clone(),
                        queued_at: Instant::now(),
                    },
                );
                cid
            } else {
                in_flight.insert(InFlightEntry {
                    client_identity: req.client_identity.clone(),
                    worker_identity: worker_id.clone(),
                    service: req.service.clone(),
                    request_id: req.request_id.clone(),
                    has_req_delim: req.has_req_delim,
                });
                req.client_identity.clone()
            };
            if let Some(m) = metrics {
                m.record_call_start(&svc_str, &req.client_identity, &req.request_id);
            }
            let fwd = build_worker_forward(
                worker_id.as_slice(),
                &forward_client_id,
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
            if let Some(idx) = req.peer_idx {
                let _ = peers[idx].dealer.send_multipart(
                    [
                        req.client_identity.as_slice(),
                        req.service.as_slice(),
                        req.request_id.as_slice(),
                        err.as_slice(),
                    ],
                    zmq::DONTWAIT,
                );
            } else {
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
            }
        } else {
            still.push_back(req);
        }
    }
    *pending = still;
    Ok(())
}

/// Expire stale corr entries and notify the original client.
pub(super) fn expire_corr(
    frontend: &Socket,
    peers: &[PeerLink],
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    now: Instant,
    pending_timeout: Duration,
    metrics: Option<&std::sync::Arc<crate::broker::service_bus::ServiceMetrics>>,
) -> Result<()> {
    let expired: Vec<Vec<u8>> = corr
        .iter()
        .filter(|(_, e)| now.duration_since(e.queued_at) > pending_timeout)
        .map(|(k, _)| k.clone())
        .collect();
    for key in expired {
        if let Some(entry) = corr.remove(&key) {
            if let Some(m) = metrics {
                let name = String::from_utf8_lossy(&entry.service);
                m.record_error(&name);
                m.record_pending_timeout(&name);
            }
            let err = build_error_body(ERR_NO_WORKER, &entry.service);
            send_error_to_target(
                frontend,
                Some(peers),
                &entry.target,
                &entry.original_client_id,
                &entry.service,
                &entry.request_id,
                &err,
            )?;
        }
    }
    Ok(())
}

/// Reclaim corr + local in-flight entries for a dead worker identity.
pub(super) fn reclaim_worker_inflight(
    frontend: &Socket,
    peers: &[PeerLink],
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    in_flight: &mut InFlightTable,
    worker_identity: &[u8],
    metrics: Option<&std::sync::Arc<crate::broker::service_bus::ServiceMetrics>>,
) -> Result<()> {
    reclaim_worker_requests(frontend, in_flight, worker_identity, metrics)?;

    let keys: Vec<Vec<u8>> = corr
        .iter()
        .filter(|(_, e)| e.worker_identity == worker_identity)
        .map(|(k, _)| k.clone())
        .collect();
    for key in keys {
        if let Some(entry) = corr.remove(&key) {
            if let Some(m) = metrics {
                let name = String::from_utf8_lossy(&entry.service);
                m.record_error(&name);
                m.record_worker_died(&name);
            }
            let err = build_error_body(ERR_WORKER_DIED, &entry.service);
            send_error_to_target(
                frontend,
                Some(peers),
                &entry.target,
                &entry.original_client_id,
                &entry.service,
                &entry.request_id,
                &err,
            )?;
        }
    }
    Ok(())
}
