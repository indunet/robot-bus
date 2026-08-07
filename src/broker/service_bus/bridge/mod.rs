//! Service-bus federation: peer-as-worker with READY_FED advertisement and hop-path anti-loop.
//!
//! Each configured peer gets a DEALER connected to the peer's service backend with identity
//! `fed/<broker_id>`. Local (and transitively reachable) services are advertised with
//! `READY_FED`; peers appear in the local [`WorkerRegistry`] as federated workers. Client
//! wire format is unchanged.

mod advertise;
mod dispatch;
mod protocol;
mod remote;

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;
use zmq::{Context as ZmqContext, Socket};

use self::advertise::{send_peer_heartbeats, sync_all_advertisements};
use self::dispatch::{
    deliver_reply, dispatch_request, expire_corr, reclaim_worker_inflight, retry_pending,
};
use self::protocol::{CMD_DISCONNECT, CMD_HEARTBEAT, CMD_READY, CMD_READY_FED, POLL_CAP_MS};
use self::remote::{
    CorrEntry, PeerLink, PendingRequest, ReplyTarget, connect_peer, is_fed_identity,
    learn_peer_broker_id, parse_corr_id, parse_fed_broker_id,
};
use super::broker::{
    ERR_NO_WORKER, InFlightTable, WorkerRegistry, build_error_body, extend_hops, hop_contains,
};
use super::{ServiceBusConfig, ServiceMetrics};

/// Run the federated service broker until `shutdown` is set.
pub fn run_federated(
    context: &ZmqContext,
    frontend: Socket,
    backend: Socket,
    config: &ServiceBusConfig,
    shutdown: &AtomicBool,
    metrics: Option<&Arc<ServiceMetrics>>,
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
    let mut in_flight = InFlightTable::new();
    let pending_timeout = Duration::from_millis(config.pending_timeout_ms);
    let max_pending = config.max_pending;

    let mut next_sweep = Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);

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
                &mut in_flight,
                &broker_id,
                max_pending,
                metrics,
            )?;
        }
        if backend_ready {
            handle_backend(
                &backend,
                &frontend,
                &mut registry,
                &client_is_req,
                &mut corr,
                &mut in_flight,
                &mut peers,
                &broker_id,
                metrics,
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
                    &mut in_flight,
                    &broker_id,
                    max_pending,
                    metrics,
                )?;
            }
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            let dead = registry.sweep_dead(now, Duration::from_millis(config.heartbeat_timeout_ms));
            for wid in dead {
                reclaim_worker_inflight(
                    &frontend,
                    &peers,
                    &mut corr,
                    &mut in_flight,
                    &wid,
                    metrics,
                )?;
            }
            sync_all_advertisements(&mut peers, &registry, &broker_id)?;
            send_peer_heartbeats(&peers)?;
            retry_pending(
                &frontend,
                &backend,
                &peers,
                &mut pending,
                &mut registry,
                &mut corr,
                &mut in_flight,
                &broker_id,
                now,
                pending_timeout,
                max_pending,
                metrics,
            )?;
            expire_corr(&frontend, &peers, &mut corr, now, pending_timeout, metrics)?;
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

fn handle_frontend(
    frontend: &Socket,
    backend: &Socket,
    registry: &mut WorkerRegistry,
    pending: &mut VecDeque<PendingRequest>,
    client_is_req: &mut HashMap<Vec<u8>, bool>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    in_flight: &mut InFlightTable,
    broker_id: &str,
    max_pending: usize,
    metrics: Option<&Arc<ServiceMetrics>>,
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
        in_flight,
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
        max_pending,
        metrics,
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
    in_flight: &mut InFlightTable,
    broker_id: &str,
    max_pending: usize,
    metrics: Option<&Arc<ServiceMetrics>>,
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
        let err = build_error_body(ERR_NO_WORKER, &svc);
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
        in_flight,
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
        max_pending,
        metrics,
    )
}

fn handle_backend(
    backend: &Socket,
    frontend: &Socket,
    registry: &mut WorkerRegistry,
    client_is_req: &HashMap<Vec<u8>, bool>,
    corr: &mut HashMap<Vec<u8>, CorrEntry>,
    in_flight: &mut InFlightTable,
    peers: &mut [PeerLink],
    broker_id: &str,
    metrics: Option<&Arc<ServiceMetrics>>,
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
                    // Only reclaim when the identity is fully gone.
                    if !registry.is_alive(worker_id) {
                        reclaim_worker_inflight(
                            frontend, peers, corr, in_flight, worker_id, metrics,
                        )?;
                    }
                } else {
                    registry.remove(worker_id);
                    reclaim_worker_inflight(frontend, peers, corr, in_flight, worker_id, metrics)?;
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
            registry.register_federated(worker_id.to_vec(), svc, now, via, origin, hop);
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
                in_flight,
                client_id,
                svc,
                req_id,
                body,
                metrics,
            )
        }
        _ => Ok(()),
    }
}
