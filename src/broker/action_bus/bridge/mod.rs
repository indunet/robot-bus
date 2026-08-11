//! Action-bus federation: peer DEALER + RemoteActions table + GoalTable-centric routing.
//!
//! Unlike service federation (one-shot corr ids), action state is keyed by `goal_id`.
//! FEEDBACK may arrive many times; only RESULT clears the goal. CANCEL and death
//! recovery also consult GoalTable.

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
    deliver_goal_reply, dispatch_goal, handle_cancel, retry_pending, send_died_results,
};
use self::protocol::{
    CMD_DISCONNECT, CMD_HEARTBEAT, CMD_READY, CMD_READY_FED, KIND_CANCEL, KIND_FEEDBACK, KIND_GOAL,
    KIND_RESULT, POLL_CAP_MS,
};
use self::remote::{
    PeerLink, PendingGoal, RemoteActions, connect_peer, is_fed_identity, learn_peer_broker_id,
    parse_fed_broker_id,
};
use super::broker::{
    GoalReply, GoalTable, WorkerRegistry, build_client_reply, build_error_body, extend_hops,
    hop_contains,
};
use super::{ActionBusConfig, ActionMetrics};

/// Run the federated action broker until `shutdown` is set.
pub fn run_federated(
    context: &ZmqContext,
    frontend: Socket,
    backend: Socket,
    config: &ActionBusConfig,
    shutdown: &AtomicBool,
    metrics: Option<&Arc<ActionMetrics>>,
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
        "action_bus_broker federation enabled\n  \
         broker_id: {broker_id}\n  \
         peers: {}",
        peers.len()
    );

    let mut registry = WorkerRegistry::new();
    let mut remote = RemoteActions::new();
    let mut goals = GoalTable::new();
    let mut pending: VecDeque<PendingGoal> = VecDeque::new();
    let mut next_sweep = Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);
    let pending_timeout = Duration::from_millis(config.pending_timeout_ms);
    let hb_timeout = Duration::from_millis(config.heartbeat_timeout_ms);

    // Round-robin cursor for remote routes.
    let mut remote_rr: HashMap<String, usize> = HashMap::new();

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
                &mut peers,
                &mut registry,
                &mut remote,
                &mut goals,
                &mut pending,
                &mut remote_rr,
                &broker_id,
                metrics,
            )?;
        }
        if backend_ready {
            handle_backend(
                &backend,
                &frontend,
                &mut peers,
                &mut registry,
                &mut remote,
                &mut goals,
                &mut pending,
                &mut remote_rr,
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
                    &mut remote,
                    &mut goals,
                    &mut pending,
                    &mut remote_rr,
                    &broker_id,
                    metrics,
                )?;
            }
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            let dead = registry.sweep_dead(now, hb_timeout);
            for (wid, act) in &dead {
                if let Some(m) = metrics {
                    m.set_workers(act, registry.worker_count(act) as u64);
                }
                let dropped = goals.drain_if(|e| e.worker_identity.as_slice() == wid.as_slice());
                send_died_results(&frontend, &backend, dropped, metrics)?;
            }
            let dead_peers = remote.sweep_dead(now, hb_timeout);
            for peer_idx in dead_peers {
                let dropped = goals.evict_peer(peer_idx);
                send_died_results(&frontend, &backend, dropped, metrics)?;
            }
            sync_all_advertisements(&mut peers, &registry, &remote, &broker_id)?;
            send_peer_heartbeats(&peers)?;
            retry_pending(
                &frontend,
                &backend,
                &mut peers,
                &mut pending,
                &mut registry,
                &mut remote,
                &mut goals,
                &mut remote_rr,
                &broker_id,
                now,
                pending_timeout,
                metrics,
            )?;
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

fn handle_frontend(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    pending: &mut VecDeque<PendingGoal>,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let frames = match frontend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("frontend recv_multipart"),
    };
    // [client_id][action][goal_id][kind][body]
    if frames.len() < 5 {
        return Ok(());
    }
    let client_id = frames[0].clone();
    let action = frames[1].clone();
    let goal_id = frames[2].clone();
    let kind = frames[3].as_slice();
    let body = frames[4].clone();

    if kind == KIND_GOAL {
        dispatch_goal(
            frontend,
            backend,
            peers,
            registry,
            remote,
            goals,
            pending,
            remote_rr,
            broker_id,
            &client_id,
            &action,
            &goal_id,
            &body,
            "",
            GoalReply::Frontend,
            metrics,
        )
    } else if kind == KIND_CANCEL {
        handle_cancel(
            frontend,
            backend,
            peers,
            goals,
            pending,
            &client_id,
            &action,
            &goal_id,
            &body,
            GoalReply::Frontend,
            metrics,
        )
    } else {
        Ok(())
    }
}

fn handle_peer_dealer(
    peer_idx: usize,
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    _remote: &mut RemoteActions,
    goals: &mut GoalTable,
    _pending: &mut VecDeque<PendingGoal>,
    _remote_rr: &mut HashMap<String, usize>,
    _broker_id: &str,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let frames = match peers[peer_idx].dealer.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("peer dealer recv_multipart"),
    };

    // Replies for goals we forwarded outbound: [action][goal_id][kind][hop][body]
    if frames.len() == 5 {
        let action = &frames[0];
        let goal_id = &frames[1];
        let kind = frames[2].as_slice();
        let body = &frames[4];
        if kind == KIND_FEEDBACK || kind == KIND_RESULT {
            return deliver_goal_reply(
                frontend, backend, peers, registry, goals, goal_id, action, kind, body, metrics,
            );
        }
    }
    Ok(())
}

fn handle_backend(
    backend: &Socket,
    frontend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    pending: &mut VecDeque<PendingGoal>,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let frames = match backend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("backend recv_multipart"),
    };
    let worker_id = frames.first().map(|v| v.as_slice()).unwrap_or(&[]);
    let now = Instant::now();

    if is_fed_identity(worker_id) {
        return handle_fed_backend_message(
            backend, frontend, peers, registry, remote, goals, pending, remote_rr, broker_id,
            &frames, now, metrics,
        );
    }

    match frames.len() {
        3 => {
            let cmd = &frames[1];
            let action = &frames[2];
            if cmd == CMD_READY {
                let act = String::from_utf8_lossy(action).into_owned();
                registry.register(worker_id.to_vec(), act, now);
                sync_all_advertisements(peers, registry, remote, broker_id)?;
            } else if cmd == CMD_HEARTBEAT {
                registry.heartbeat(worker_id, now);
            } else if cmd == CMD_DISCONNECT {
                registry.remove(worker_id);
                let dropped = goals.drain_if(|e| e.worker_identity.as_slice() == worker_id);
                send_died_results(frontend, backend, dropped, metrics)?;
                sync_all_advertisements(peers, registry, remote, broker_id)?;
            }
            Ok(())
        }
        6 => {
            let action = &frames[2];
            let goal_id = &frames[3];
            let kind = frames[4].as_slice();
            let body = &frames[5];
            if kind != KIND_FEEDBACK && kind != KIND_RESULT {
                return Ok(());
            }
            if goals.get(goal_id).is_some() {
                deliver_goal_reply(
                    frontend, backend, peers, registry, goals, goal_id, action, kind, body, metrics,
                )?;
            } else {
                let client_id = &frames[1];
                let reply = build_client_reply(client_id, action, goal_id, kind, body);
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send feedback/result")?;
                if kind == KIND_RESULT {
                    registry.release_worker(worker_id);
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_fed_backend_message(
    backend: &Socket,
    frontend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    pending: &mut VecDeque<PendingGoal>,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    frames: &[Vec<u8>],
    now: Instant,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let identity = frames[0].as_slice();
    let Some(via) = parse_fed_broker_id(identity) else {
        return Ok(());
    };

    match frames.len() {
        3 => {
            let cmd = &frames[1];
            let action = String::from_utf8_lossy(&frames[2]).into_owned();
            let Some(peer_idx) = learn_peer_broker_id(peers, &via) else {
                return Ok(());
            };
            if cmd == CMD_HEARTBEAT {
                // Bootstrap a direct route if READY_FED never arrived.
                if !remote.heartbeat_peer(peer_idx, &action, now) {
                    remote.upsert(action, peer_idx, via.clone(), via, now);
                    sync_all_advertisements(peers, registry, remote, broker_id)?;
                }
            } else if cmd == CMD_READY {
                remote.upsert(action, peer_idx, via.clone(), via, now);
                sync_all_advertisements(peers, registry, remote, broker_id)?;
            } else if cmd == CMD_DISCONNECT {
                remote.remove_action_peer(peer_idx, &action);
                sync_all_advertisements(peers, registry, remote, broker_id)?;
            }
            Ok(())
        }
        5 if frames[1].as_slice() == CMD_READY_FED => {
            let action = String::from_utf8_lossy(&frames[2]).into_owned();
            let origin = String::from_utf8_lossy(&frames[3]).into_owned();
            let hop = String::from_utf8_lossy(&frames[4]).into_owned();
            if hop_contains(&hop, broker_id) {
                return Ok(());
            }
            let Some(peer_idx) = learn_peer_broker_id(peers, &via) else {
                return Ok(());
            };
            remote.upsert(action, peer_idx, origin, hop, now);
            sync_all_advertisements(peers, registry, remote, broker_id)?;
            Ok(())
        }
        6 => {
            let action = frames[1].clone();
            let goal_id = frames[2].clone();
            let kind = frames[3].as_slice();
            let hop = String::from_utf8_lossy(&frames[4]).into_owned();
            let body = frames[5].clone();

            let mut hops = hop;
            if !hop_contains(&hops, &via) {
                hops = extend_hops(&hops, &via);
            }
            if hop_contains(&hops, broker_id) {
                if kind == KIND_GOAL {
                    let err = build_error_body(protocol::ERR_NO_WORKER, &action);
                    let _ = backend.send_multipart(
                        [
                            identity,
                            action.as_slice(),
                            goal_id.as_slice(),
                            KIND_RESULT,
                            b"",
                            err.as_slice(),
                        ],
                        zmq::DONTWAIT,
                    );
                }
                return Ok(());
            }
            hops = extend_hops(&hops, broker_id);

            let reply = GoalReply::FedBackend {
                identity: identity.to_vec(),
            };

            if kind == KIND_GOAL {
                dispatch_goal(
                    frontend, backend, peers, registry, remote, goals, pending, remote_rr,
                    broker_id, identity, &action, &goal_id, &body, &hops, reply, metrics,
                )?;
            } else if kind == KIND_CANCEL {
                handle_cancel(
                    frontend, backend, peers, goals, pending, identity, &action, &goal_id, &body,
                    reply, metrics,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
