//! Service-bus federation: peer-as-worker with READY_FED advertisement and hop-path anti-loop.
//!
//! Each configured peer gets a DEALER connected to the peer's service backend with identity
//! `fed/<broker_id>`. Local (and transitively reachable) services are advertised with
//! `READY_FED`; peers appear in the local [`WorkerRegistry`] as federated workers. Client
//! wire format is unchanged.

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;
use zmq::{Context as ZmqContext, Socket, SocketType};

use super::broker::{
    build_client_reply, build_error_body, build_worker_forward, extend_hops, hop_contains,
    WorkerRegistry,
};
use super::ServiceBusConfig;

const CMD_READY: &[u8] = b"READY";
const CMD_READY_FED: &[u8] = b"READY_FED";
const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

const FED_ID_PREFIX: &str = "fed/";
const FED_REQ_PREFIX: &str = "fedreq/";

const POLL_CAP_MS: i64 = 200;
const MAX_PENDING: usize = 64;
const PENDING_TIMEOUT: Duration = Duration::from_secs(5);

struct PeerLink {
    dealer: Socket,
    /// Peer broker id (from config or learned from reverse READY_FED).
    peer_broker_id: Option<String>,
    /// service -> (origin, hop) we currently advertise on this link.
    advertised: HashMap<String, (String, String)>,
}

enum ReplyTarget {
    Frontend { has_req_delim: bool },
    Peer { peer_idx: usize },
}

struct CorrEntry {
    target: ReplyTarget,
    original_client_id: Vec<u8>,
    queued_at: Instant,
}

struct PendingRequest {
    client_identity: Vec<u8>,
    service: Vec<u8>,
    request_id: Vec<u8>,
    body: Vec<u8>,
    has_req_delim: bool,
    hop_path: String,
    peer_idx: Option<usize>,
    queued_at: Instant,
}

/// Run the federated service broker until `shutdown` is set.
pub fn run_federated(
    context: &ZmqContext,
    frontend: Socket,
    backend: Socket,
    config: &ServiceBusConfig,
    shutdown: &AtomicBool,
) -> Result<()> {
    let broker_id = if config.broker_id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        config.broker_id.clone()
    };

    let mut peers = Vec::with_capacity(config.peers.len());
    for peer in &config.peers {
        peers.push(connect_peer(
            context,
            &peer.backend,
            &broker_id,
            peer.broker_id.as_str(),
            config.snd_hwm,
            config.rcv_hwm,
        )?);
    }

    println!(
        "service_bus_broker federation enabled\n  \
         broker_id: {broker_id}\n  \
         peers: {}",
        peers.len()
    );

    let mut registry = WorkerRegistry::new();
    let mut pending: VecDeque<PendingRequest> = VecDeque::new();
    let mut client_is_req: HashMap<Vec<u8>, bool> = HashMap::new();
    let mut corr: HashMap<Vec<u8>, CorrEntry> = HashMap::new();

    let mut next_sweep =
        Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        let sweep_in = next_sweep.saturating_duration_since(Instant::now());
        let timeout_ms = (sweep_in.as_millis() as i64).min(POLL_CAP_MS).max(0);

        let mut items = Vec::with_capacity(2 + peers.len());
        items.push(frontend.as_poll_item(zmq::POLLIN));
        items.push(backend.as_poll_item(zmq::POLLIN));
        for link in &peers {
            items.push(link.dealer.as_poll_item(zmq::POLLIN));
        }
        zmq::poll(&mut items, timeout_ms).context("poll")?;

        let frontend_ready = items[0].get_revents().contains(zmq::POLLIN);
        let backend_ready = items[1].get_revents().contains(zmq::POLLIN);
        let peer_ready: Vec<bool> = items[2..]
            .iter()
            .map(|it| it.get_revents().contains(zmq::POLLIN))
            .collect();
        drop(items);

        if frontend_ready {
            handle_frontend(
                &frontend,
                &backend,
                &mut registry,
                &mut pending,
                &mut client_is_req,
                &mut corr,
                &broker_id,
            )?;
        }
        if backend_ready {
            handle_backend(
                &backend,
                &frontend,
                &mut registry,
                &client_is_req,
                &mut corr,
                &mut peers,
                &broker_id,
            )?;
        }
        for (i, ready) in peer_ready.into_iter().enumerate() {
            if ready {
                handle_peer_dealer(
                    i,
                    &frontend,
                    &backend,
                    &mut peers,
                    &mut registry,
                    &mut pending,
                    &mut corr,
                    &broker_id,
                )?;
            }
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            let _ = registry.sweep_dead(
                now,
                Duration::from_millis(config.heartbeat_timeout_ms),
            );
            sync_all_advertisements(&mut peers, &registry, &broker_id)?;
            send_peer_heartbeats(&peers)?;
            retry_pending(
                &frontend,
                &backend,
                &peers,
                &mut pending,
                &mut registry,
                &mut corr,
                &broker_id,
                now,
            )?;
            expire_corr(&mut corr, now);
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

fn connect_peer(
    context: &ZmqContext,
    backend: &str,
    broker_id: &str,
    peer_broker_id: &str,
    snd_hwm: i32,
    rcv_hwm: i32,
) -> Result<PeerLink> {
    let dealer = context
        .socket(SocketType::DEALER)
        .context("create federation DEALER")?;
    dealer.set_linger(0).context("set linger")?;
    dealer.set_sndhwm(snd_hwm).context("set sndhwm")?;
    dealer.set_rcvhwm(rcv_hwm).context("set rcvhwm")?;
    dealer.set_immediate(true).context("set immediate")?;
    let identity = format!("{FED_ID_PREFIX}{broker_id}");
    dealer
        .set_identity(identity.as_bytes())
        .context("set federation identity")?;
    dealer
        .connect(backend)
        .with_context(|| format!("connect federation DEALER to {backend}"))?;
    Ok(PeerLink {
        dealer,
        peer_broker_id: if peer_broker_id.is_empty() {
            None
        } else {
            Some(peer_broker_id.to_string())
        },
        advertised: HashMap::new(),
    })
}

fn parse_fed_broker_id(identity: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(identity).ok()?;
    s.strip_prefix(FED_ID_PREFIX)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

fn is_fed_identity(identity: &[u8]) -> bool {
    parse_fed_broker_id(identity).is_some()
}

fn make_corr_id(hops: &str) -> Vec<u8> {
    let corr = Uuid::new_v4().simple().to_string();
    format!("{FED_REQ_PREFIX}{corr}/{hops}").into_bytes()
}

fn parse_corr_id(id: &[u8]) -> Option<(String, String)> {
    let s = std::str::from_utf8(id).ok()?;
    let rest = s.strip_prefix(FED_REQ_PREFIX)?;
    let (corr, hops) = rest.split_once('/')?;
    Some((corr.to_string(), hops.to_string()))
}

fn handle_frontend(
    frontend: &Socket,
    backend: &Socket,
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    client_is_req: &mut HashMap<Vec<u8>, bool>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    broker_id: &str,
) -> Result<()> {
    let frames = match frontend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("frontend recv_multipart"),
    };
    if frames.len() < 4 {
        return Ok(());
    }
    let has_delim = frames.len() == 5 && frames[1].is_empty();
    let off = if has_delim { 2 } else { 1 };
    let client_id = &frames[0];
    let svc = &frames[off];
    let req_id = &frames[off + 1];
    let body = &frames[off + 2];
    if svc.is_empty() {
        return Ok(());
    }
    let svc_str = match std::str::from_utf8(svc) {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(()),
    };
    client_is_req.insert(client_id.to_vec(), has_delim);

    dispatch_request(
        frontend,
        backend,
        None,
        registry,
        pending,
        corr,
        broker_id,
        client_id,
        svc,
        &svc_str,
        req_id,
        body,
        "",
        ReplyTarget::Frontend {
            has_req_delim: has_delim,
        },
        None,
        has_delim,
    )
}

fn handle_peer_dealer(
    peer_idx: usize,
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    broker_id: &str,
) -> Result<()> {
    let frames = match peers[peer_idx].dealer.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("peer dealer recv_multipart"),
    };
    // Inbound request as worker: [client_id][svc][req_id][body]
    if frames.len() != 4 {
        return Ok(());
    }
    let client_id = frames[0].clone();
    let svc = frames[1].clone();
    let req_id = frames[2].clone();
    let body = frames[3].clone();
    if svc.is_empty() {
        return Ok(());
    }
    let svc_str = match std::str::from_utf8(&svc) {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(()),
    };

    let mut hops = if let Some((_, h)) = parse_corr_id(&client_id) {
        h
    } else {
        String::new()
    };
    if let Some(pid) = peers[peer_idx].peer_broker_id.clone() {
        if !hop_contains(&hops, &pid) {
            hops = extend_hops(&hops, &pid);
        }
    }
    if hop_contains(&hops, broker_id) {
        let err = build_error_body(&svc);
        let _ = peers[peer_idx].dealer.send_multipart(
            [
                client_id.as_slice(),
                svc.as_slice(),
                req_id.as_slice(),
                err.as_slice(),
            ],
            zmq::DONTWAIT,
        );
        return Ok(());
    }
    hops = extend_hops(&hops, broker_id);

    dispatch_request(
        frontend,
        backend,
        Some(peers),
        registry,
        pending,
        corr,
        broker_id,
        &client_id,
        &svc,
        &svc_str,
        &req_id,
        &body,
        &hops,
        ReplyTarget::Peer { peer_idx },
        Some(peer_idx),
        false,
    )
}

fn dispatch_request(
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

fn handle_backend(
    backend: &Socket,
    frontend: &Socket,
    registry: &mut WorkerRegistry,
    client_is_req: &HashMap<Vec<u8>, bool>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    peers: &mut [PeerLink],
    broker_id: &str,
) -> Result<()> {
    let frames = match backend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("backend recv_multipart"),
    };
    let worker_id = frames.first().map(|v| v.as_slice()).unwrap_or(&[]);
    let now = Instant::now();

    match frames.len() {
        3 => {
            let cmd = &frames[1];
            let svc = &frames[2];
            if cmd == CMD_READY {
                if is_fed_identity(worker_id) {
                    if let Some(via) = parse_fed_broker_id(worker_id) {
                        let svc_str = String::from_utf8_lossy(svc).into_owned();
                        learn_peer_broker_id(peers, &via);
                        registry.register_federated(
                            worker_id.to_vec(),
                            svc_str,
                            now,
                            via.clone(),
                            via.clone(),
                            via,
                        );
                        sync_all_advertisements(peers, registry, broker_id)?;
                    }
                } else {
                    let svc_str = String::from_utf8_lossy(svc).into_owned();
                    registry.register(worker_id.to_vec(), svc_str, now);
                    sync_all_advertisements(peers, registry, broker_id)?;
                }
            } else if cmd == CMD_HEARTBEAT {
                registry.heartbeat(worker_id, now);
            } else if cmd == CMD_DISCONNECT {
                let svc_str = String::from_utf8_lossy(svc).into_owned();
                if is_fed_identity(worker_id) {
                    registry.remove_service_binding(worker_id, &svc_str);
                } else {
                    registry.remove(worker_id);
                }
                sync_all_advertisements(peers, registry, broker_id)?;
            }
            Ok(())
        }
        5 if frames[1] == CMD_READY_FED => {
            let svc = String::from_utf8_lossy(&frames[2]).into_owned();
            let origin = String::from_utf8_lossy(&frames[3]).into_owned();
            let hop = String::from_utf8_lossy(&frames[4]).into_owned();
            let Some(via) = parse_fed_broker_id(worker_id) else {
                return Ok(());
            };
            if hop_contains(&hop, broker_id) {
                return Ok(());
            }
            learn_peer_broker_id(peers, &via);
            registry.register_federated(
                worker_id.to_vec(),
                svc,
                now,
                via,
                origin,
                hop,
            );
            sync_all_advertisements(peers, registry, broker_id)?;
            Ok(())
        }
        5 => {
            registry.release_worker(worker_id);
            let client_id = &frames[1];
            let svc = &frames[2];
            let req_id = &frames[3];
            let body = &frames[4];
            deliver_reply(
                frontend,
                peers,
                client_is_req,
                corr,
                client_id,
                svc,
                req_id,
                body,
            )
        }
        _ => Ok(()),
    }
}

/// Fill in a peer link's broker id when learned from a reverse registration.
fn learn_peer_broker_id(peers: &mut [PeerLink], via: &str) {
    if peers
        .iter()
        .any(|p| p.peer_broker_id.as_deref() == Some(via))
    {
        return;
    }
    if let Some(link) = peers.iter_mut().find(|p| p.peer_broker_id.is_none()) {
        link.peer_broker_id = Some(via.to_string());
    }
}

fn deliver_reply(
    frontend: &Socket,
    peers: &mut [PeerLink],
    client_is_req: &HashMap<Vec<u8>, bool>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    client_id: &[u8],
    svc: &[u8],
    req_id: &[u8],
    body: &[u8],
) -> Result<()> {
    if let Some(entry) = corr.remove(client_id) {
        match entry.target {
            ReplyTarget::Frontend { has_req_delim } => {
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

    let has_delim = client_is_req.get(client_id).copied().unwrap_or(false);
    let reply = build_client_reply(client_id, svc, req_id, body, has_delim);
    frontend
        .send_multipart(reply, 0)
        .context("frontend send reply")?;
    Ok(())
}

fn sync_all_advertisements(
    peers: &mut [PeerLink],
    registry: &WorkerRegistry,
    broker_id: &str,
) -> Result<()> {
    let snapshot = registry.advertise_snapshot(broker_id);
    for link in peers.iter_mut() {
        sync_peer_advertisement(link, &snapshot, broker_id)?;
    }
    Ok(())
}

fn sync_peer_advertisement(
    link: &mut PeerLink,
    snapshot: &[(String, String, String, String)],
    broker_id: &str,
) -> Result<()> {
    let peer_id = link.peer_broker_id.clone();
    let mut desired: HashMap<String, (String, String)> = HashMap::new();

    for (svc, origin, hop, via) in snapshot {
        if !via.is_empty() {
            match &peer_id {
                Some(pid) if via == pid || hop_contains(hop, pid) => continue,
                None => continue, // wait until we know peer id before re-advertising
                _ => {}
            }
        } else if let Some(pid) = &peer_id {
            // Local offering: still skip if somehow hop already contains peer.
            if hop_contains(hop, pid) {
                continue;
            }
        }

        desired
            .entry(svc.clone())
            .and_modify(|(o, h)| {
                if origin == broker_id {
                    *o = origin.clone();
                    *h = hop.clone();
                }
            })
            .or_insert_with(|| (origin.clone(), hop.clone()));
    }

    let stale: Vec<String> = link
        .advertised
        .keys()
        .filter(|s| !desired.contains_key(s.as_str()))
        .cloned()
        .collect();
    for svc in stale {
        let _ = link
            .dealer
            .send_multipart([CMD_DISCONNECT, svc.as_bytes()], zmq::DONTWAIT);
        link.advertised.remove(&svc);
    }

    for (svc, (origin, hop)) in &desired {
        let need_ready = match link.advertised.get(svc) {
            Some((o, h)) if o == origin && h == hop => false,
            _ => true,
        };
        if need_ready {
            let _ = link.dealer.send_multipart(
                [
                    CMD_READY_FED,
                    svc.as_bytes(),
                    origin.as_bytes(),
                    hop.as_bytes(),
                ],
                zmq::DONTWAIT,
            );
            link.advertised
                .insert(svc.clone(), (origin.clone(), hop.clone()));
        }
    }
    Ok(())
}

fn send_peer_heartbeats(peers: &[PeerLink]) -> Result<()> {
    for link in peers {
        for svc in link.advertised.keys() {
            let _ = link
                .dealer
                .send_multipart([CMD_HEARTBEAT, svc.as_bytes()], zmq::DONTWAIT);
        }
    }
    Ok(())
}

fn retry_pending(
    frontend: &Socket,
    backend: &Socket,
    peers: &[PeerLink],
    pending: &mut VecDeque<PendingRequest>,
    registry: &mut WorkerRegistry,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    broker_id: &str,
    now: Instant,
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

fn expire_corr(corr: &mut HashMap<Vec<u8>, CorrEntry>, now: Instant) {
    corr.retain(|_, e| now.duration_since(e.queued_at) <= PENDING_TIMEOUT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fed_identity_parse() {
        assert_eq!(
            parse_fed_broker_id(b"fed/broker-a").as_deref(),
            Some("broker-a")
        );
        assert!(parse_fed_broker_id(b"worker-1").is_none());
    }

    #[test]
    fn corr_id_roundtrip() {
        let id = make_corr_id("a,b");
        let (corr, hops) = parse_corr_id(&id).unwrap();
        assert!(!corr.is_empty());
        assert_eq!(hops, "a,b");
    }
}
