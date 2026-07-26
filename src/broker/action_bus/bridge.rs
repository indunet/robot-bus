//! Action-bus federation: peer DEALER + RemoteActions table + GoalTable-centric routing.
//!
//! Unlike service federation (one-shot corr ids), action state is keyed by `goal_id`.
//! FEEDBACK may arrive many times; only RESULT clears the goal. CANCEL and death
//! recovery also consult GoalTable.

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;
use zmq::{Context as ZmqContext, Socket, SocketType};

use super::broker::{
    build_client_reply, build_error_body, build_worker_cancel, build_worker_goal, extend_hops,
    hop_contains, GoalReply, GoalTable, WorkerRegistry,
};
use super::ActionBusConfig;

const CMD_READY: &[u8] = b"READY";
const CMD_READY_FED: &[u8] = b"READY_FED";
const CMD_HEARTBEAT: &[u8] = b"HEARTBEAT";
const CMD_DISCONNECT: &[u8] = b"DISCONNECT";

const KIND_GOAL: &[u8] = b"GOAL";
const KIND_FEEDBACK: &[u8] = b"FEEDBACK";
const KIND_RESULT: &[u8] = b"RESULT";
const KIND_CANCEL: &[u8] = b"CANCEL";

const ERR_NO_WORKER: &[u8] = b"NO_WORKER";
const ERR_WORKER_DIED: &[u8] = b"WORKER_DIED";
const ERR_NO_GOAL: &[u8] = b"NO_GOAL";

const FED_ID_PREFIX: &str = "fed/";
const POLL_CAP_MS: i64 = 200;
const MAX_PENDING: usize = 64;

struct PeerLink {
    dealer: Socket,
    peer_broker_id: Option<String>,
    /// action -> (origin, hop) currently advertised on this link.
    advertised: HashMap<String, (String, String)>,
}

#[derive(Clone, Debug)]
struct RemoteRoute {
    peer_idx: usize,
    origin_broker_id: String,
    hop_path: String,
    last_heartbeat: Instant,
}

/// action_name -> remote routes.
struct RemoteActions {
    by_action: HashMap<String, Vec<RemoteRoute>>,
}

impl RemoteActions {
    fn new() -> Self {
        Self {
            by_action: HashMap::new(),
        }
    }

    fn upsert(
        &mut self,
        action: String,
        peer_idx: usize,
        origin: String,
        hop: String,
        now: Instant,
    ) {
        let list = self.by_action.entry(action).or_default();
        if let Some(r) = list.iter_mut().find(|r| r.peer_idx == peer_idx) {
            r.origin_broker_id = origin;
            r.hop_path = hop;
            r.last_heartbeat = now;
        } else {
            list.push(RemoteRoute {
                peer_idx,
                origin_broker_id: origin,
                hop_path: hop,
                last_heartbeat: now,
            });
        }
    }

    fn heartbeat_peer(&mut self, peer_idx: usize, action: &str, now: Instant) {
        if let Some(list) = self.by_action.get_mut(action) {
            for r in list.iter_mut().filter(|r| r.peer_idx == peer_idx) {
                r.last_heartbeat = now;
            }
        }
    }

    fn remove_action_peer(&mut self, peer_idx: usize, action: &str) {
        if let Some(list) = self.by_action.get_mut(action) {
            list.retain(|r| r.peer_idx != peer_idx);
            if list.is_empty() {
                self.by_action.remove(action);
            }
        }
    }

    fn sweep_dead(&mut self, now: Instant, timeout: Duration) -> Vec<usize> {
        let mut touched_peers = Vec::new();
        let actions: Vec<String> = self.by_action.keys().cloned().collect();
        for act in actions {
            if let Some(list) = self.by_action.get_mut(&act) {
                let before = list.len();
                list.retain(|r| {
                    let alive = now.duration_since(r.last_heartbeat) <= timeout;
                    if !alive {
                        touched_peers.push(r.peer_idx);
                    }
                    alive
                });
                if list.is_empty() {
                    self.by_action.remove(&act);
                } else if list.len() != before {
                    // peer routes changed
                }
            }
        }
        touched_peers.sort_unstable();
        touched_peers.dedup();
        touched_peers
    }

    /// Offerings for re-advertisement: (action, origin, hop, via_peer_idx).
    fn advertise_snapshot(&self, self_id: &str) -> Vec<(String, String, String, usize)> {
        let mut out = Vec::new();
        for (act, list) in &self.by_action {
            for r in list {
                out.push((
                    act.clone(),
                    r.origin_broker_id.clone(),
                    extend_hops(&r.hop_path, self_id),
                    r.peer_idx,
                ));
            }
        }
        out
    }
}

struct PendingGoal {
    client_identity: Vec<u8>,
    action: Vec<u8>,
    goal_id: Vec<u8>,
    body: Vec<u8>,
    hop_path: String,
    reply: GoalReply,
    queued_at: Instant,
}

/// Run the federated action broker until `shutdown` is set.
pub fn run_federated(
    context: &ZmqContext,
    frontend: Socket,
    backend: Socket,
    config: &ActionBusConfig,
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
        "action_bus_broker federation enabled\n  \
         broker_id: {broker_id}\n  \
         peers: {}",
        peers.len()
    );

    let mut registry = WorkerRegistry::new();
    let mut remote = RemoteActions::new();
    let mut goals = GoalTable::new();
    let mut pending: VecDeque<PendingGoal> = VecDeque::new();
    let mut next_sweep =
        Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);
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
                )?;
            }
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            let dead = registry.sweep_dead(now, hb_timeout);
            for wid in &dead {
                let dropped = goals.drain_if(|e| e.worker_identity.as_slice() == wid.as_slice());
                send_died_results(&frontend, &backend, dropped)?;
            }
            let dead_peers = remote.sweep_dead(now, hb_timeout);
            for peer_idx in dead_peers {
                let dropped = goals.evict_peer(peer_idx);
                send_died_results(&frontend, &backend, dropped)?;
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
            )?;
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
        .context("create action federation DEALER")?;
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
        .with_context(|| format!("connect action federation DEALER to {backend}"))?;
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

fn find_peer_idx(peers: &[PeerLink], broker_id: &str) -> Option<usize> {
    peers
        .iter()
        .position(|p| p.peer_broker_id.as_deref() == Some(broker_id))
}

fn learn_peer_broker_id(peers: &mut [PeerLink], via: &str) -> Option<usize> {
    if let Some(idx) = find_peer_idx(peers, via) {
        return Some(idx);
    }
    if let Some((idx, link)) = peers
        .iter_mut()
        .enumerate()
        .find(|(_, p)| p.peer_broker_id.is_none())
    {
        link.peer_broker_id = Some(via.to_string());
        return Some(idx);
    }
    None
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
        )
    } else if kind == KIND_CANCEL {
        handle_cancel(
            frontend,
            backend,
            peers,
            goals,
            &client_id,
            &action,
            &goal_id,
            &body,
            GoalReply::Frontend,
        )
    } else {
        Ok(())
    }
}

fn handle_peer_dealer(
    _peer_idx: usize,
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    _remote: &mut RemoteActions,
    goals: &mut GoalTable,
    _pending: &mut VecDeque<PendingGoal>,
    _remote_rr: &mut HashMap<String, usize>,
    _broker_id: &str,
) -> Result<()> {
    let frames = match peers[_peer_idx].dealer.recv_multipart(0) {
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
                frontend, backend, peers, registry, goals, goal_id, action, kind, body,
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
            backend,
            frontend,
            peers,
            registry,
            remote,
            goals,
            pending,
            remote_rr,
            broker_id,
            &frames,
            now,
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
                let dropped =
                    goals.drain_if(|e| e.worker_identity.as_slice() == worker_id);
                send_died_results(frontend, backend, dropped)?;
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
                    frontend, backend, peers, registry, goals, goal_id, action, kind, body,
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
                remote.heartbeat_peer(peer_idx, &action, now);
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
                    let err = build_error_body(ERR_NO_WORKER, &action);
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
                    frontend,
                    backend,
                    peers,
                    registry,
                    remote,
                    goals,
                    pending,
                    remote_rr,
                    broker_id,
                    identity,
                    &action,
                    &goal_id,
                    &body,
                    &hops,
                    reply,
                )?;
            } else if kind == KIND_CANCEL {
                handle_cancel(
                    frontend,
                    backend,
                    peers,
                    goals,
                    identity,
                    &action,
                    &goal_id,
                    &body,
                    reply,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn select_remote<'a>(
    remote: &'a RemoteActions,
    remote_rr: &mut HashMap<String, usize>,
    action: &str,
    hop_path: &str,
) -> Option<&'a RemoteRoute> {
    let list = remote.by_action.get(action)?;
    let candidates: Vec<usize> = list
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            !hop_contains(hop_path, &r.origin_broker_id)
                && !r
                    .hop_path
                    .split(',')
                    .any(|h| !h.is_empty() && hop_contains(hop_path, h))
        })
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let cursor = remote_rr.entry(action.to_string()).or_insert(0);
    let pick = candidates[*cursor % candidates.len()];
    *cursor = (*cursor + 1) % candidates.len();
    Some(&list[pick])
}

fn dispatch_goal(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    pending: &mut VecDeque<PendingGoal>,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    body: &[u8],
    hop_path: &str,
    reply: GoalReply,
) -> Result<()> {
    let act_str = match std::str::from_utf8(action) {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(()),
    };
    let hops_for_select = if hop_path.is_empty() {
        broker_id.to_string()
    } else {
        hop_path.to_string()
    };

    // Local first.
    if registry.worker_count(&act_str) > 0 {
        if let Some(worker_id) = registry.select_worker(&act_str) {
            goals.insert_full(
                goal_id.to_vec(),
                client_id.to_vec(),
                worker_id.clone(),
                action.to_vec(),
                clone_reply(&reply),
                None,
            );
            // Forward to local worker with a synthetic client id encoding goal_id
            // so replies can always be matched — actually workers echo client_id.
            // We pass client_id as goal bookkeeping; deliver uses GoalTable by goal_id.
            let fwd = build_worker_goal(&worker_id, client_id, action, goal_id, body);
            backend.send_multipart(fwd, 0).context("backend send goal")?;
            return Ok(());
        }
    }

    // Remote.
    if let Some(route) = select_remote(remote, remote_rr, &act_str, &hops_for_select) {
        let peer_idx = route.peer_idx;
        let out_hop = extend_hops(&hops_for_select, broker_id);
        goals.insert_full(
            goal_id.to_vec(),
            client_id.to_vec(),
            Vec::new(),
            action.to_vec(),
            clone_reply(&reply),
            Some(peer_idx),
        );
        let _ = peers[peer_idx].dealer.send_multipart(
            [
                action,
                goal_id,
                KIND_GOAL,
                out_hop.as_bytes(),
                body,
            ],
            zmq::DONTWAIT,
        );
        return Ok(());
    }

    if pending.len() < MAX_PENDING {
        pending.push_back(PendingGoal {
            client_identity: client_id.to_vec(),
            action: action.to_vec(),
            goal_id: goal_id.to_vec(),
            body: body.to_vec(),
            hop_path: hops_for_select,
            reply: clone_reply(&reply),
            queued_at: Instant::now(),
        });
    } else {
        send_error_result(
            frontend,
            backend,
            peers,
            &reply,
            client_id,
            action,
            goal_id,
            ERR_NO_WORKER,
        )?;
    }
    Ok(())
}

fn clone_reply(r: &GoalReply) -> GoalReply {
    match r {
        GoalReply::Frontend => GoalReply::Frontend,
        GoalReply::FedBackend { identity } => GoalReply::FedBackend {
            identity: identity.clone(),
        },
    }
}

fn handle_cancel(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    goals: &GoalTable,
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    body: &[u8],
    reply: GoalReply,
) -> Result<()> {
    if let Some(entry) = goals.get(goal_id) {
        if let Some(peer_idx) = entry.via_peer {
            let hop = ""; // cancel follows established goal; hop not needed for routing
            let _ = peers[peer_idx].dealer.send_multipart(
                [action, goal_id, KIND_CANCEL, hop.as_bytes(), body],
                zmq::DONTWAIT,
            );
        } else if !entry.worker_identity.is_empty() {
            let fwd = build_worker_cancel(
                &entry.worker_identity,
                client_id,
                action,
                goal_id,
                body,
            );
            backend
                .send_multipart(fwd, 0)
                .context("backend send cancel")?;
        }
    } else {
        send_error_result(
            frontend,
            backend,
            peers,
            &reply,
            client_id,
            action,
            goal_id,
            ERR_NO_GOAL,
        )?;
    }
    Ok(())
}

fn deliver_goal_reply(
    frontend: &Socket,
    backend: &Socket,
    peers: &[PeerLink],
    registry: &mut WorkerRegistry,
    goals: &mut GoalTable,
    goal_id: &[u8],
    action: &[u8],
    kind: &[u8],
    body: &[u8],
) -> Result<()> {
    let Some(entry) = goals.get(goal_id) else {
        return Ok(());
    };
    match &entry.reply {
        GoalReply::Frontend => {
            let reply = build_client_reply(
                &entry.client_identity,
                action,
                goal_id,
                kind,
                body,
            );
            frontend
                .send_multipart(reply, 0)
                .context("frontend send feedback/result")?;
        }
        GoalReply::FedBackend { identity } => {
            // Reply toward the peer that sent us the goal (via backend ROUTER).
            let _ = backend.send_multipart(
                [
                    identity.as_slice(),
                    action,
                    goal_id,
                    kind,
                    b"", // hop unused on return
                    body,
                ],
                zmq::DONTWAIT,
            );
        }
    }

    if kind == KIND_RESULT {
        if let Some(entry) = goals.remove(goal_id) {
            if !entry.worker_identity.is_empty() {
                registry.release_worker(&entry.worker_identity);
            }
        }
    }
    let _ = peers; // peer dealer replies already landed here via handle_peer_dealer
    Ok(())
}

fn send_error_result(
    frontend: &Socket,
    backend: &Socket,
    peers: &[PeerLink],
    reply: &GoalReply,
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    err_prefix: &[u8],
) -> Result<()> {
    let err = build_error_body(err_prefix, action);
    match reply {
        GoalReply::Frontend => {
            let msg = build_client_reply(client_id, action, goal_id, KIND_RESULT, &err);
            frontend
                .send_multipart(msg, 0)
                .context("frontend send error result")?;
        }
        GoalReply::FedBackend { identity } => {
            let _ = backend.send_multipart(
                [
                    identity.as_slice(),
                    action,
                    goal_id,
                    KIND_RESULT,
                    b"",
                    err.as_slice(),
                ],
                zmq::DONTWAIT,
            );
        }
    }
    let _ = peers;
    Ok(())
}

fn send_died_results(
    frontend: &Socket,
    backend: &Socket,
    dropped: Vec<super::broker::GoalEntry>,
) -> Result<()> {
    for e in dropped {
        let err = build_error_body(ERR_WORKER_DIED, &e.action);
        match &e.reply {
            GoalReply::Frontend => {
                let reply = build_client_reply(
                    &e.client_identity,
                    &e.action,
                    &e.goal_id,
                    KIND_RESULT,
                    &err,
                );
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send worker-died")?;
            }
            GoalReply::FedBackend { identity } => {
                let _ = backend.send_multipart(
                    [
                        identity.as_slice(),
                        e.action.as_slice(),
                        e.goal_id.as_slice(),
                        KIND_RESULT,
                        b"",
                        err.as_slice(),
                    ],
                    zmq::DONTWAIT,
                );
            }
        }
    }
    Ok(())
}

fn sync_all_advertisements(
    peers: &mut [PeerLink],
    registry: &WorkerRegistry,
    remote: &RemoteActions,
    broker_id: &str,
) -> Result<()> {
    let mut snapshot: Vec<(String, String, String, Option<usize>)> = Vec::new();
    for act in registry.action_names() {
        snapshot.push((act, broker_id.to_string(), broker_id.to_string(), None));
    }
    for (act, origin, hop, via) in remote.advertise_snapshot(broker_id) {
        snapshot.push((act, origin, hop, Some(via)));
    }

    for (idx, link) in peers.iter_mut().enumerate() {
        sync_peer_advertisement(link, idx, &snapshot, broker_id)?;
    }
    Ok(())
}

fn sync_peer_advertisement(
    link: &mut PeerLink,
    peer_idx: usize,
    snapshot: &[(String, String, String, Option<usize>)],
    broker_id: &str,
) -> Result<()> {
    let peer_id = link.peer_broker_id.clone();
    let mut desired: HashMap<String, (String, String)> = HashMap::new();

    for (act, origin, hop, via) in snapshot {
        if let Some(via_idx) = via {
            if *via_idx == peer_idx {
                continue; // don't advertise back to source peer
            }
        }
        if let Some(ref pid) = peer_id {
            if hop_contains(hop, pid) {
                continue;
            }
            if via.is_some() && origin != broker_id {
                // already filtered by via_idx; also skip if hop contains peer
            }
        } else if via.is_some() {
            continue; // wait until peer id known before re-advertising remotes
        }

        desired
            .entry(act.clone())
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
    for act in stale {
        let _ = link
            .dealer
            .send_multipart([CMD_DISCONNECT, act.as_bytes()], zmq::DONTWAIT);
        link.advertised.remove(&act);
    }

    for (act, (origin, hop)) in &desired {
        let need = match link.advertised.get(act) {
            Some((o, h)) if o == origin && h == hop => false,
            _ => true,
        };
        if need {
            let _ = link.dealer.send_multipart(
                [
                    CMD_READY_FED,
                    act.as_bytes(),
                    origin.as_bytes(),
                    hop.as_bytes(),
                ],
                zmq::DONTWAIT,
            );
            link.advertised
                .insert(act.clone(), (origin.clone(), hop.clone()));
        }
    }
    Ok(())
}

fn send_peer_heartbeats(peers: &[PeerLink]) -> Result<()> {
    for link in peers {
        for act in link.advertised.keys() {
            let _ = link
                .dealer
                .send_multipart([CMD_HEARTBEAT, act.as_bytes()], zmq::DONTWAIT);
        }
    }
    Ok(())
}

fn retry_pending(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    pending: &mut VecDeque<PendingGoal>,
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    now: Instant,
    pending_timeout: Duration,
) -> Result<()> {
    let mut still = VecDeque::new();
    while let Some(req) = pending.pop_front() {
        let act_str = match std::str::from_utf8(&req.action) {
            Ok(s) => s.to_string(),
            Err(_) => continue,
        };
        let mut dispatched = false;
        if registry.worker_count(&act_str) > 0 {
            if let Some(worker_id) = registry.select_worker(&act_str) {
                goals.insert_full(
                    req.goal_id.clone(),
                    req.client_identity.clone(),
                    worker_id.clone(),
                    req.action.clone(),
                    clone_reply(&req.reply),
                    None,
                );
                let fwd = build_worker_goal(
                    &worker_id,
                    &req.client_identity,
                    &req.action,
                    &req.goal_id,
                    &req.body,
                );
                backend
                    .send_multipart(fwd, 0)
                    .context("backend send pending goal")?;
                dispatched = true;
            }
        }
        if !dispatched {
            if let Some(route) = select_remote(remote, remote_rr, &act_str, &req.hop_path) {
                let peer_idx = route.peer_idx;
                let out_hop = extend_hops(&req.hop_path, broker_id);
                goals.insert_full(
                    req.goal_id.clone(),
                    req.client_identity.clone(),
                    Vec::new(),
                    req.action.clone(),
                    clone_reply(&req.reply),
                    Some(peer_idx),
                );
                let _ = peers[peer_idx].dealer.send_multipart(
                    [
                        req.action.as_slice(),
                        req.goal_id.as_slice(),
                        KIND_GOAL,
                        out_hop.as_bytes(),
                        req.body.as_slice(),
                    ],
                    zmq::DONTWAIT,
                );
                dispatched = true;
            }
        }
        if !dispatched {
            if now.duration_since(req.queued_at) > pending_timeout {
                send_error_result(
                    frontend,
                    backend,
                    peers,
                    &req.reply,
                    &req.client_identity,
                    &req.action,
                    &req.goal_id,
                    ERR_NO_WORKER,
                )?;
            } else {
                still.push_back(req);
            }
        }
    }
    *pending = still;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_actions_upsert_and_select() {
        let mut r = RemoteActions::new();
        let now = Instant::now();
        r.upsert("act.x".into(), 0, "a".into(), "a".into(), now);
        r.upsert("act.x".into(), 1, "c".into(), "b,c".into(), now);
        let mut rr = HashMap::new();
        let sel = select_remote(&r, &mut rr, "act.x", "a").unwrap();
        assert_eq!(sel.peer_idx, 1);
        assert!(select_remote(&r, &mut rr, "act.x", "a,c").is_none());
    }

    #[test]
    fn hop_helpers() {
        assert!(hop_contains("a,b", "b"));
        assert_eq!(extend_hops("a", "b"), "a,b");
    }
}
