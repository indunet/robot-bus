//! Action-bus federation: peer links + RemoteActions table + GoalTable-centric routing.
//!
//! Unlike service federation (one-shot corr ids), action state is keyed by `goal_id`.
//! FEEDBACK may arrive many times; only RESULT clears the goal. CANCEL and death
//! recovery also consult GoalTable.
//!
//! A configured peer gets a DEALER connected to the peer's action backend with identity
//! `fed/<broker_id>`. Local (and transitively reachable) actions are advertised with
//! `READY_FED`; goals and cancels go out over the same DEALER.
//!
//! **One-sided join.** When the peer supports it (`ActionPeer::join`) the same DEALER also
//! carries the other direction, so the peer needs no configuration: we announce ourselves
//! with `JOIN`, the peer answers `JOIN_ACK` and advertises its actions (`READY_FED`) and
//! sends goals / cancels (through its backend ROUTER) over the link. A broker that never
//! configured any peer runs the same loop, so a joining broker is served the moment it
//! connects.

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
    CMD_DISCONNECT, CMD_HEARTBEAT, CMD_JOIN, CMD_JOIN_ACK, CMD_READY, CMD_READY_FED, DEFER,
    KIND_CANCEL, KIND_FEEDBACK, KIND_GOAL, KIND_RESULT, POLL_CAP_MS,
};
pub(super) use self::remote::is_fed_identity;
use self::remote::{
    LinkKind, PeerLink, PendingGoal, RemoteActions, connect_peer, find_in_link, find_join_link,
    learn_peer_broker_id, parse_fed_broker_id,
};
use super::broker::{
    GoalReply, GoalTable, WorkerRegistry, build_client_reply, build_error_body, extend_hops,
    hop_contains,
};
use super::{ActionBusConfig, ActionMetrics};

/// How often a joining link announces itself.
const JOIN_INTERVAL: Duration = Duration::from_secs(1);

struct State {
    broker_id: String,
    epoch: String,
    registry: WorkerRegistry,
    remote: RemoteActions,
    goals: GoalTable,
    pending: VecDeque<PendingGoal>,
    remote_rr: HashMap<String, usize>,
    /// Configured links (`Out`) followed by links that joined us (`In`, appended as they
    /// appear and never removed, so indices stay valid).
    peers: Vec<PeerLink>,
    timeout: Duration,
}

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
            peer.join,
            config.snd_hwm,
            config.rcv_hwm,
        )?);
    }

    log::info!(
        "action_bus federation ready: broker_id={broker_id} peers={}",
        peers.len()
    );

    let mut st = State {
        broker_id,
        epoch: Uuid::new_v4().simple().to_string(),
        registry: WorkerRegistry::new(),
        remote: RemoteActions::new(),
        goals: GoalTable::new(),
        pending: VecDeque::new(),
        remote_rr: HashMap::new(),
        peers,
        timeout: Duration::from_millis(config.heartbeat_timeout_ms),
    };
    let pending_timeout = Duration::from_millis(config.pending_timeout_ms);
    let join_interval = JOIN_INTERVAL.min(Duration::from_millis(config.heartbeat_interval_ms));
    let mut next_sweep = Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        let now = Instant::now();
        st.send_joins(&backend, now, join_interval);

        let sweep_in = next_sweep.saturating_duration_since(now);
        let timeout_ms = (sweep_in.as_millis() as i64).min(POLL_CAP_MS).max(0);

        let out_links: Vec<usize> = st
            .peers
            .iter()
            .enumerate()
            .filter(|(_, p)| p.dealer().is_some())
            .map(|(i, _)| i)
            .collect();
        let mut items = Vec::with_capacity(2 + out_links.len());
        items.push(frontend.as_poll_item(zmq::POLLIN));
        items.push(backend.as_poll_item(zmq::POLLIN));
        for &i in &out_links {
            if let Some(dealer) = st.peers[i].dealer() {
                items.push(dealer.as_poll_item(zmq::POLLIN));
            }
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
            st.handle_frontend(&frontend, &backend, metrics)?;
        }
        if backend_ready {
            st.handle_backend(&backend, &frontend, metrics)?;
        }
        for (slot, ready) in peer_ready.into_iter().enumerate() {
            if ready {
                st.handle_peer_dealer(out_links[slot], &frontend, &backend, metrics)?;
            }
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            st.sweep(&frontend, &backend, now, pending_timeout, metrics)?;
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

impl State {
    /// Announce ourselves on every joining link.
    fn send_joins(&mut self, backend: &Socket, now: Instant, interval: Duration) {
        for link in &mut self.peers {
            if link.join && link.dealer().is_some() && now >= link.next_join {
                link.next_join = now + interval;
                link.send(backend, &[CMD_JOIN, self.epoch.as_bytes()]);
            }
        }
    }

    fn sync(&mut self, backend: &Socket) -> Result<()> {
        sync_all_advertisements(
            &mut self.peers,
            backend,
            &self.registry,
            &self.remote,
            &self.broker_id,
        )
    }

    fn handle_frontend(
        &mut self,
        frontend: &Socket,
        backend: &Socket,
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
        let client_id = &frames[0];
        let action = &frames[1];
        let goal_id = &frames[2];
        let kind = frames[3].as_slice();
        let body = &frames[4];

        if kind == KIND_GOAL {
            dispatch_goal(
                frontend,
                backend,
                &mut self.peers,
                &mut self.registry,
                &mut self.remote,
                &mut self.goals,
                &mut self.pending,
                &mut self.remote_rr,
                &self.broker_id,
                client_id,
                action,
                goal_id,
                body,
                "",
                GoalReply::Frontend,
                metrics,
            )
        } else if kind == KIND_CANCEL {
            handle_cancel(
                frontend,
                backend,
                &mut self.peers,
                &self.goals,
                &mut self.pending,
                client_id,
                action,
                goal_id,
                body,
                GoalReply::Frontend,
                metrics,
            )
        } else {
            Ok(())
        }
    }

    /// A goal or cancel another broker sends us to serve.
    ///
    /// `hop` is the sender's hop path, `via` its broker id, `identity` what we use as the
    /// client identity for bookkeeping, `reply` how answers travel back.
    fn serve_peer_goal(
        &mut self,
        frontend: &Socket,
        backend: &Socket,
        via: &str,
        identity: &[u8],
        reply: GoalReply,
        frames: [&[u8]; 5],
        metrics: Option<&Arc<ActionMetrics>>,
    ) -> Result<()> {
        let [action, goal_id, kind, hop, body] = frames;
        let mut hops = String::from_utf8_lossy(hop).into_owned();
        if !hop_contains(&hops, via) {
            hops = extend_hops(&hops, via);
        }
        if hop_contains(&hops, &self.broker_id) {
            if kind == KIND_GOAL {
                let err = build_error_body(protocol::ERR_NO_WORKER, action);
                send_reply(&self.peers, backend, &reply, action, goal_id, KIND_RESULT, &err);
            }
            return Ok(());
        }
        hops = extend_hops(&hops, &self.broker_id);

        if kind == KIND_GOAL {
            dispatch_goal(
                frontend,
                backend,
                &mut self.peers,
                &mut self.registry,
                &mut self.remote,
                &mut self.goals,
                &mut self.pending,
                &mut self.remote_rr,
                &self.broker_id,
                identity,
                action,
                goal_id,
                body,
                &hops,
                reply,
                metrics,
            )
        } else if kind == KIND_CANCEL {
            handle_cancel(
                frontend,
                backend,
                &mut self.peers,
                &self.goals,
                &mut self.pending,
                identity,
                action,
                goal_id,
                body,
                reply,
                metrics,
            )
        } else {
            Ok(())
        }
    }

    /// Frames from a configured peer's DEALER.
    fn handle_peer_dealer(
        &mut self,
        peer_idx: usize,
        frontend: &Socket,
        backend: &Socket,
        metrics: Option<&Arc<ActionMetrics>>,
    ) -> Result<()> {
        let frames = {
            let Some(dealer) = self.peers[peer_idx].dealer() else {
                return Ok(());
            };
            match dealer.recv_multipart(0) {
                Ok(f) => f,
                Err(zmq::Error::EAGAIN) => return Ok(()),
                Err(e) => return Err(e).context("peer dealer recv_multipart"),
            }
        };
        let now = Instant::now();
        let Some(first) = frames.first() else {
            return Ok(());
        };

        if first.as_slice() == CMD_JOIN_ACK {
            return self.handle_join_ack(peer_idx, &frames, now, frontend, backend, metrics);
        }

        let link = &mut self.peers[peer_idx];
        if link.join {
            if link.passive {
                return Ok(());
            }
            link.last_seen = now;
            let via = link.peer_broker_id.clone();
            // Control frames of the joined broker: [cmd][action]… (first frame is a command).
            match first.as_slice() {
                f if f == CMD_READY_FED && frames.len() == 4 => {
                    let action = String::from_utf8_lossy(&frames[1]).into_owned();
                    let origin = String::from_utf8_lossy(&frames[2]).into_owned();
                    let hop = String::from_utf8_lossy(&frames[3]).into_owned();
                    if hop_contains(&hop, &self.broker_id) {
                        return Ok(());
                    }
                    if self.remote.has_route(&action, peer_idx, &origin, &hop) {
                        self.remote.heartbeat_peer(peer_idx, &action, now);
                    } else {
                        self.remote.upsert(action, peer_idx, origin, hop, now);
                        self.sync(backend)?;
                    }
                    return Ok(());
                }
                f if f == CMD_HEARTBEAT && frames.len() == 2 => {
                    let action = String::from_utf8_lossy(&frames[1]).into_owned();
                    self.remote.heartbeat_peer(peer_idx, &action, now);
                    return Ok(());
                }
                f if f == CMD_DISCONNECT && frames.len() == 2 => {
                    let action = String::from_utf8_lossy(&frames[1]).into_owned();
                    self.remote.remove_action_peer(peer_idx, &action);
                    self.sync(backend)?;
                    return Ok(());
                }
                _ => {}
            }
            // A goal / cancel the joined broker sends us: [action][goal_id][kind][hop][body].
            if frames.len() == 5 && (frames[2] == KIND_GOAL || frames[2] == KIND_CANCEL) {
                let Some(via) = via else {
                    return Ok(());
                };
                let identity = format!("{}{via}", protocol::FED_ID_PREFIX).into_bytes();
                return self.serve_peer_goal(
                    frontend,
                    backend,
                    &via,
                    &identity,
                    GoalReply::FedDealer { peer_idx },
                    [
                        &frames[0], &frames[1], &frames[2], &frames[3], &frames[4],
                    ],
                    metrics,
                );
            }
        }

        // Replies for goals we forwarded outbound: [action][goal_id][kind][hop][body]
        if frames.len() == 5 {
            let action = &frames[0];
            let goal_id = &frames[1];
            let kind = frames[2].as_slice();
            let body = &frames[4];
            if kind == KIND_FEEDBACK || kind == KIND_RESULT {
                return deliver_goal_reply(
                    frontend,
                    backend,
                    &self.peers,
                    &mut self.registry,
                    &mut self.goals,
                    goal_id,
                    action,
                    kind,
                    body,
                    metrics,
                );
            }
        }
        Ok(())
    }

    /// `JOIN_ACK id epoch [DEFER]` from the broker we joined.
    fn handle_join_ack(
        &mut self,
        peer_idx: usize,
        frames: &[Vec<u8>],
        now: Instant,
        frontend: &Socket,
        backend: &Socket,
        metrics: Option<&Arc<ActionMetrics>>,
    ) -> Result<()> {
        if frames.len() < 3 || !self.peers[peer_idx].join {
            return Ok(());
        }
        let peer_id = String::from_utf8_lossy(&frames[1]).into_owned();
        let epoch = String::from_utf8_lossy(&frames[2]).into_owned();
        let defer = frames.get(3).is_some_and(|f| f == DEFER);

        let link = &mut self.peers[peer_idx];
        if link.peer_broker_id.is_none() {
            link.peer_broker_id = Some(peer_id.clone());
        }
        let was_alive = link.alive;
        let was_passive = link.passive;
        let restarted = link.epoch.as_deref().is_some_and(|old| old != epoch);
        link.alive = true;
        link.last_seen = now;
        link.epoch = Some(epoch);
        link.passive = defer;
        if !was_alive {
            log::info!("action federation: joined broker {peer_id}");
        }
        if defer && !was_passive {
            log::info!(
                "action federation: broker {peer_id} also configured this broker; its link \
                 carries the traffic, keeping this connection idle"
            );
        }

        if restarted || (defer && !was_passive) {
            // The peer forgot everything we told it / its actions are no longer ours.
            link.advertised.clear();
            self.drop_link_routes(peer_idx, frontend, backend, metrics)?;
        }
        if restarted || (!defer && (was_passive || !was_alive)) {
            self.sync(backend)?;
        }
        Ok(())
    }

    /// Forget the actions a peer offered over link `peer_idx` and fail its goals.
    fn drop_link_routes(
        &mut self,
        peer_idx: usize,
        frontend: &Socket,
        backend: &Socket,
        metrics: Option<&Arc<ActionMetrics>>,
    ) -> Result<()> {
        self.remote.remove_peer(peer_idx);
        let dropped = self.goals.evict_peer(peer_idx);
        send_died_results(frontend, backend, &self.peers, dropped, metrics)
    }

    fn handle_backend(
        &mut self,
        backend: &Socket,
        frontend: &Socket,
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
            return self.handle_fed_backend_message(backend, frontend, &frames, now, metrics);
        }

        match frames.len() {
            3 => {
                let cmd = &frames[1];
                let action = &frames[2];
                if cmd == CMD_READY || (cmd == CMD_HEARTBEAT && !self.registry.is_alive(worker_id))
                {
                    let act = String::from_utf8_lossy(action).into_owned();
                    self.registry.register(worker_id.to_vec(), act.clone(), now);
                    if let Some(m) = metrics {
                        m.set_workers(&act, self.registry.worker_count(&act) as u64);
                    }
                    self.sync(backend)?;
                } else if cmd == CMD_HEARTBEAT {
                    self.registry.heartbeat(worker_id, now);
                } else if cmd == CMD_DISCONNECT {
                    let act = self.registry.remove(worker_id);
                    let dropped = self
                        .goals
                        .drain_if(|e| e.worker_identity.as_slice() == worker_id);
                    send_died_results(frontend, backend, &self.peers, dropped, metrics)?;
                    if let (Some(m), Some(act)) = (metrics, act) {
                        m.set_workers(&act, self.registry.worker_count(&act) as u64);
                    }
                    self.sync(backend)?;
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
                if self.goals.get(goal_id).is_some() {
                    deliver_goal_reply(
                        frontend,
                        backend,
                        &self.peers,
                        &mut self.registry,
                        &mut self.goals,
                        goal_id,
                        action,
                        kind,
                        body,
                        metrics,
                    )?;
                } else {
                    let client_id = &frames[1];
                    let reply = build_client_reply(client_id, action, goal_id, kind, body);
                    frontend
                        .send_multipart(reply, 0)
                        .context("frontend send feedback/result")?;
                    if kind == KIND_RESULT {
                        self.registry.release_worker(worker_id);
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn handle_fed_backend_message(
        &mut self,
        backend: &Socket,
        frontend: &Socket,
        frames: &[Vec<u8>],
        now: Instant,
        metrics: Option<&Arc<ActionMetrics>>,
    ) -> Result<()> {
        let identity = frames[0].as_slice();
        let Some(via) = parse_fed_broker_id(identity) else {
            return Ok(());
        };

        match frames.len() {
            3 if frames[1] == CMD_JOIN => {
                self.handle_join(backend, identity, &frames[2], now)?;
                Ok(())
            }
            3 => {
                let cmd = &frames[1];
                let action = String::from_utf8_lossy(&frames[2]).into_owned();
                let Some(peer_idx) = learn_peer_broker_id(&mut self.peers, &via) else {
                    return Ok(());
                };
                if cmd == CMD_HEARTBEAT {
                    // Bootstrap a direct route if READY_FED never arrived.
                    if !self.remote.heartbeat_peer(peer_idx, &action, now) {
                        self.remote.upsert(action, peer_idx, via.clone(), via, now);
                        self.sync(backend)?;
                    }
                } else if cmd == CMD_READY {
                    self.remote.upsert(action, peer_idx, via.clone(), via, now);
                    self.sync(backend)?;
                } else if cmd == CMD_DISCONNECT {
                    self.remote.remove_action_peer(peer_idx, &action);
                    self.sync(backend)?;
                }
                Ok(())
            }
            5 if frames[1].as_slice() == CMD_READY_FED => {
                let action = String::from_utf8_lossy(&frames[2]).into_owned();
                let origin = String::from_utf8_lossy(&frames[3]).into_owned();
                let hop = String::from_utf8_lossy(&frames[4]).into_owned();
                if hop_contains(&hop, &self.broker_id) {
                    return Ok(());
                }
                let Some(peer_idx) = learn_peer_broker_id(&mut self.peers, &via) else {
                    return Ok(());
                };
                if self.remote.has_route(&action, peer_idx, &origin, &hop) {
                    self.remote.heartbeat_peer(peer_idx, &action, now);
                } else {
                    self.remote.upsert(action, peer_idx, origin, hop, now);
                    self.sync(backend)?;
                }
                Ok(())
            }
            6 => {
                let kind = frames[3].as_slice();
                if kind == KIND_FEEDBACK || kind == KIND_RESULT {
                    // Answer to a goal we sent through a link that joined us.
                    return deliver_goal_reply(
                        frontend,
                        backend,
                        &self.peers,
                        &mut self.registry,
                        &mut self.goals,
                        &frames[2],
                        &frames[1],
                        kind,
                        &frames[5],
                        metrics,
                    );
                }
                let reply = GoalReply::FedBackend {
                    identity: identity.to_vec(),
                };
                self.serve_peer_goal(
                    frontend,
                    backend,
                    &via,
                    identity,
                    reply,
                    [
                        &frames[1], &frames[2], &frames[3], &frames[4], &frames[5],
                    ],
                    metrics,
                )
            }
            _ => Ok(()),
        }
    }

    /// `JOIN epoch` from a broker that joins us.
    fn handle_join(
        &mut self,
        backend: &Socket,
        identity: &[u8],
        epoch: &[u8],
        now: Instant,
    ) -> Result<()> {
        let Some(via) = parse_fed_broker_id(identity) else {
            return Ok(());
        };
        let epoch = String::from_utf8_lossy(epoch).into_owned();

        // Both brokers configured each other: the link belongs to the smaller id.
        if self.broker_id.as_str() < via.as_str() && find_join_link(&self.peers, &via).is_some() {
            let _ = backend.send_multipart(
                [
                    identity,
                    CMD_JOIN_ACK,
                    self.broker_id.as_bytes(),
                    self.epoch.as_bytes(),
                    DEFER,
                ],
                zmq::DONTWAIT,
            );
            return Ok(());
        }

        let (idx, changed) = match find_in_link(&self.peers, identity) {
            Some(idx) => {
                let link = &mut self.peers[idx];
                let restarted = link.epoch.as_deref() != Some(epoch.as_str());
                let revived = !link.alive;
                link.alive = true;
                link.last_seen = now;
                link.epoch = Some(epoch);
                (idx, restarted || revived)
            }
            None => {
                log::info!("action federation: broker {via} joined");
                let now = Instant::now();
                self.peers.push(PeerLink {
                    kind: LinkKind::In(identity.to_vec()),
                    peer_broker_id: Some(via),
                    advertised: HashMap::new(),
                    join: true,
                    passive: false,
                    alive: true,
                    last_seen: now,
                    epoch: Some(epoch),
                    next_join: now,
                });
                (self.peers.len() - 1, true)
            }
        };
        let _ = backend.send_multipart(
            [
                identity,
                CMD_JOIN_ACK,
                self.broker_id.as_bytes(),
                self.epoch.as_bytes(),
            ],
            zmq::DONTWAIT,
        );
        if changed {
            // It knows nothing of us (new, restarted or expired): advertise everything.
            self.peers[idx].advertised.clear();
            self.sync(backend)?;
        }
        Ok(())
    }

    /// Periodic work: dead workers / routes, link expiry, advertisement refresh, pending retry.
    fn sweep(
        &mut self,
        frontend: &Socket,
        backend: &Socket,
        now: Instant,
        pending_timeout: Duration,
        metrics: Option<&Arc<ActionMetrics>>,
    ) -> Result<()> {
        let dead = self.registry.sweep_dead(now, self.timeout);
        for (wid, act) in &dead {
            if let Some(m) = metrics {
                m.set_workers(act, self.registry.worker_count(act) as u64);
            }
            let dropped = self
                .goals
                .drain_if(|e| e.worker_identity.as_slice() == wid.as_slice());
            send_died_results(frontend, backend, &self.peers, dropped, metrics)?;
        }
        let dead_peers = self.remote.sweep_dead(now, self.timeout);
        for peer_idx in dead_peers {
            let dropped = self.goals.evict_peer(peer_idx);
            send_died_results(frontend, backend, &self.peers, dropped, metrics)?;
        }

        // Links whose peer went quiet.
        for idx in 0..self.peers.len() {
            let link = &mut self.peers[idx];
            if !link.alive || now.duration_since(link.last_seen) <= self.timeout {
                continue;
            }
            match link.kind {
                LinkKind::Out(_) if link.join => {
                    log::warn!(
                        "action federation: broker {} stopped answering",
                        link.peer_broker_id.as_deref().unwrap_or("?")
                    );
                    link.alive = false;
                    link.passive = false;
                    link.epoch = None;
                    link.advertised.clear();
                    self.drop_link_routes(idx, frontend, backend, metrics)?;
                }
                LinkKind::In(_) => {
                    log::warn!(
                        "action federation: broker {} left",
                        link.peer_broker_id.as_deref().unwrap_or("?")
                    );
                    link.alive = false;
                    link.advertised.clear();
                }
                LinkKind::Out(_) => {}
            }
        }

        self.sync(backend)?;
        send_peer_heartbeats(&self.peers, backend)?;
        retry_pending(
            frontend,
            backend,
            &mut self.peers,
            &mut self.pending,
            &mut self.registry,
            &mut self.remote,
            &mut self.goals,
            &mut self.remote_rr,
            &self.broker_id,
            now,
            pending_timeout,
            metrics,
        )
    }
}

/// Send a RESULT / FEEDBACK-style frame set along `reply` (no goal-table lookup).
fn send_reply(
    peers: &[PeerLink],
    backend: &Socket,
    reply: &GoalReply,
    action: &[u8],
    goal_id: &[u8],
    kind: &[u8],
    body: &[u8],
) {
    match reply {
        GoalReply::Frontend => {}
        GoalReply::FedBackend { identity } => {
            let _ = backend.send_multipart(
                [identity.as_slice(), action, goal_id, kind, b"", body],
                zmq::DONTWAIT,
            );
        }
        GoalReply::FedDealer { peer_idx } => {
            peers[*peer_idx].send(backend, &[action, goal_id, kind, b"", body]);
        }
    }
}
