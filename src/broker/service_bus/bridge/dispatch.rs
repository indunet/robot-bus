//! Request dispatch, reply delivery, and pending / corr-id bookkeeping.

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use zmq::Socket;

use super::protocol::{MAX_PENDING, PENDING_TIMEOUT};
use super::remote::{
    is_fed_identity, make_corr_id, CorrEntry, PeerLink, PendingRequest, ReplyTarget,
};
use super::super::broker::{
    build_client_reply, build_error_body, build_worker_forward, WorkerRegistry,
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

pub(super) fn dispatch_request(
    frontend: &Socket,
    backend: &Socket,
    peers: Option<&mut [PeerLink]>,
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
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
                    queued_at: Instant::now(),
                },
            );
            cid
        } else {
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
    } else if pending.len() < MAX_PENDING {
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
        let err = build_error_body(svc);
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

    if let Some(m) = metrics {
        if let Ok(svc_str) = std::str::from_utf8(svc) {
            m.record_call_ok(svc_str, client_id, req_id);
        }
    }
    let has_delim = client_is_req.get(client_id).copied().unwrap_or(false);
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
    broker_id: &str,
    now: Instant,
    metrics: Option<&std::sync::Arc<crate::broker::service_bus::ServiceMetrics>>,
) -> Result<()> {
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
                        queued_at: Instant::now(),
                    },
                );
                cid
            } else {
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
        } else if now.duration_since(req.queued_at) > PENDING_TIMEOUT {
            if let Some(m) = metrics {
                m.record_error(&svc_str);
            }
            let err = build_error_body(&req.service);
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

pub(super) fn expire_corr(corr: &mut HashMap<Vec<u8>, CorrEntry>, now: Instant) {
    corr.retain(|_, e| now.duration_since(e.queued_at) <= PENDING_TIMEOUT);
}
