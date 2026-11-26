//! Service-bus federation: peer links with READY_FED advertisement and hop-path anti-loop.
//!
//! A configured peer gets a DEALER connected to the peer's service backend with identity
//! `fed/<broker_id>`. Local (and transitively reachable) services are advertised with
//! `READY_FED`; the peer registers them as federated workers and calls them as before.
//!
//! **One-sided join.** When the peer supports it (`ServicePeer::join`), the same DEALER
//! also carries the other direction, so the peer needs no configuration: we announce
//! ourselves with `JOIN`, the peer answers `JOIN_ACK` and advertises its services over the
//! link (`READY_FED` through its backend ROUTER). We register them under the identity
//! `fedlink/<peer>` and call them with `FED_CALL`; replies come back as `FED_REPLY`.
//! Client wire format is unchanged. A broker that never configured any peer runs the same
//! loop, so a joining broker is served the moment it connects.

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
use self::protocol::{
    CMD_DISCONNECT, CMD_FED_CALL, CMD_FED_REPLY, CMD_HEARTBEAT, CMD_JOIN, CMD_JOIN_ACK, CMD_READY,
    CMD_READY_FED, DEFER, POLL_CAP_MS,
};
pub(super) use self::remote::is_fed_identity;
use self::remote::{
    CorrEntry, LinkKind, PeerLink, PendingRequest, ReplyTarget, connect_peer, find_in_link,
    find_join_link, learn_peer_broker_id, parse_corr_id, parse_fed_broker_id,
};
use super::broker::{
    ERR_NO_WORKER, InFlightTable, WorkerRegistry, build_error_body, extend_hops, hop_contains,
};
use super::{ServiceBusConfig, ServiceMetrics};

/// How often a joining link announces itself.
const JOIN_INTERVAL: Duration = Duration::from_secs(1);

struct State {
    broker_id: String,
    epoch: String,
    registry: WorkerRegistry,
    pending: VecDeque<PendingRequest>,
    client_is_req: HashMap<Vec<u8>, bool>,
    corr: HashMap<Vec<u8>, CorrEntry>,
    in_flight: InFlightTable,
    /// Configured links (`Out`) followed by links that joined us (`In`, appended as they
    /// appear and never removed, so indices stay valid).
    peers: Vec<PeerLink>,
    max_pending: usize,
    timeout: Duration,
}

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
            peer.join,
            config.snd_hwm,
            config.rcv_hwm,
        )?);
    }

    log::info!(
        "service_bus federation ready: broker_id={broker_id} peers={}",
        peers.len()
    );

    let mut st = State {
        broker_id,
        epoch: Uuid::new_v4().simple().to_string(),
        registry: WorkerRegistry::new(),
        pending: VecDeque::new(),
        client_is_req: HashMap::new(),
        corr: HashMap::new(),
        in_flight: InFlightTable::new(),
        peers,
        max_pending: config.max_pending,
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
        sync_all_advertisements(&mut self.peers, backend, &self.registry, &self.broker_id)
    }

    fn handle_frontend(
        &mut self,
        frontend: &Socket,
        backend: &Socket,
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
        self.client_is_req.insert(client_id.to_vec(), has_delim);

        dispatch_request(
            frontend,
            backend,
            &self.peers,
            &mut self.registry,
            &mut self.pending,
            &mut self.corr,
            &mut self.in_flight,
            &self.broker_id,
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
            self.max_pending,
            metrics,
        )
    }

    /// A request another broker sends us to serve (`[client_id][svc][req_id][body]`, where
    /// `client_id` is the correlation id carrying the hop path).
    fn serve_peer_request(
        &mut self,
        peer_idx: usize,
        frontend: &Socket,
        backend: &Socket,
        client_id: &[u8],
        svc: &[u8],
        req_id: &[u8],
        body: &[u8],
        metrics: Option<&Arc<ServiceMetrics>>,
    ) -> Result<()> {
        if svc.is_empty() {
            return Ok(());
        }
        let svc_str = match std::str::from_utf8(svc) {
            Ok(s) => s.to_string(),
            Err(_) => return Ok(()),
        };

        let mut hops = if let Some((_, h)) = parse_corr_id(client_id) {
            h
        } else {
            String::new()
        };
        if let Some(pid) = self.peers[peer_idx].peer_broker_id.clone() {
            if !hop_contains(&hops, &pid) {
                hops = extend_hops(&hops, &pid);
            }
        }
        if hop_contains(&hops, &self.broker_id) {
            let err = build_error_body(ERR_NO_WORKER, svc);
            self.peers[peer_idx].send_reply(backend, client_id, svc, req_id, &err);
            return Ok(());
        }
        hops = extend_hops(&hops, &self.broker_id);

        dispatch_request(
            frontend,
            backend,
            &self.peers,
            &mut self.registry,
            &mut self.pending,
            &mut self.corr,
            &mut self.in_flight,
            &self.broker_id,
            client_id,
            svc,
            &svc_str,
            req_id,
            body,
            &hops,
            ReplyTarget::Peer { peer_idx },
            Some(peer_idx),
            false,
            self.max_pending,
            metrics,
        )
    }

    /// Frames from a configured peer's DEALER.
    fn handle_peer_dealer(
        &mut self,
        peer_idx: usize,
        frontend: &Socket,
        backend: &Socket,
        metrics: Option<&Arc<ServiceMetrics>>,
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
        // Everything below comes from the peer's side of a link we joined.
        if self.peers[peer_idx].join {
            if self.peers[peer_idx].passive {
                return Ok(());
            }
            self.peers[peer_idx].last_seen = now;
            let Some(link_id) = self.peers[peer_idx].link_identity() else {
                return Ok(());
            };
            match first.as_slice() {
                f if f == CMD_READY_FED && frames.len() == 4 => {
                    let svc = String::from_utf8_lossy(&frames[1]).into_owned();
                    let origin = String::from_utf8_lossy(&frames[2]).into_owned();
                    let hop = String::from_utf8_lossy(&frames[3]).into_owned();
                    self.register_link_service(&link_id, svc, origin, hop, now, backend)?;
                    return Ok(());
                }
                f if f == CMD_DISCONNECT && frames.len() == 2 => {
                    let svc = String::from_utf8_lossy(&frames[1]).into_owned();
                    self.registry.remove_service_binding(&link_id, &svc);
                    if !self.registry.is_alive(&link_id) {
                        reclaim_worker_inflight(
                            frontend,
                            backend,
                            &self.peers,
                            &mut self.corr,
                            &mut self.in_flight,
                            &link_id,
                            metrics,
                        )?;
                    }
                    self.sync(backend)?;
                    return Ok(());
                }
                f if f == CMD_HEARTBEAT => {
                    self.registry.heartbeat(&link_id, now);
                    return Ok(());
                }
                f if f == CMD_FED_REPLY && frames.len() == 5 => {
                    return deliver_reply(
                        frontend,
                        backend,
                        &self.peers,
                        &self.client_is_req,
                        &mut self.corr,
                        &mut self.in_flight,
                        &frames[1],
                        &frames[2],
                        &frames[3],
                        &frames[4],
                        metrics,
                    );
                }
                _ => {}
            }
        }

        // Inbound request as worker: [client_id][svc][req_id][body]
        if frames.len() != 4 {
            return Ok(());
        }
        self.serve_peer_request(
            peer_idx, frontend, backend, &frames[0], &frames[1], &frames[2], &frames[3], metrics,
        )
    }

    /// `JOIN_ACK id epoch [DEFER]` from the broker we joined.
    fn handle_join_ack(
        &mut self,
        peer_idx: usize,
        frames: &[Vec<u8>],
        now: Instant,
        frontend: &Socket,
        backend: &Socket,
        metrics: Option<&Arc<ServiceMetrics>>,
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
            log::info!("service federation: joined broker {peer_id}");
        }
        if defer && !was_passive {
            log::info!(
                "service federation: broker {peer_id} also configured this broker; its link \
                 carries the traffic, keeping this connection idle"
            );
        }

        if restarted || (defer && !was_passive) {
            // The peer forgot everything we told it / its services are no longer ours.
            link.advertised.clear();
            self.drop_link_routes(peer_idx, frontend, backend, metrics)?;
        }
        if restarted || (!defer && (was_passive || !was_alive)) {
            self.sync(backend)?;
        }
        Ok(())
    }

    /// Forget the services a joined broker offered over our joining link.
    fn drop_link_routes(
        &mut self,
        peer_idx: usize,
        frontend: &Socket,
        backend: &Socket,
        metrics: Option<&Arc<ServiceMetrics>>,
    ) -> Result<()> {
        let Some(link_id) = self.peers[peer_idx].link_identity() else {
            return Ok(());
        };
        if self.registry.is_alive(&link_id) {
            self.registry.remove(&link_id);
            reclaim_worker_inflight(
                frontend,
                backend,
                &self.peers,
                &mut self.corr,
                &mut self.in_flight,
                &link_id,
                metrics,
            )?;
        }
        Ok(())
    }

    /// Register (or refresh) a service the joined broker advertised to us.
    fn register_link_service(
        &mut self,
        link_id: &[u8],
        svc: String,
        origin: String,
        hop: String,
        now: Instant,
        backend: &Socket,
    ) -> Result<()> {
        let Some(via) = remote::parse_link_broker_id(link_id) else {
            return Ok(());
        };
        if hop_contains(&hop, &self.broker_id) {
            return Ok(());
        }
        let same = self.registry.federated_meta(link_id, &svc)
            == Some((via.clone(), origin.clone(), hop.clone()));
        if same {
            self.registry.heartbeat(link_id, now);
        } else {
            self.registry
                .register_federated(link_id.to_vec(), svc, now, via, origin, hop);
            self.sync(backend)?;
        }
        Ok(())
    }

    fn handle_backend(
        &mut self,
        backend: &Socket,
        frontend: &Socket,
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
            3 if frames[1] == CMD_JOIN => {
                self.handle_join(backend, worker_id, &frames[2], now)?;
                Ok(())
            }
            3 => {
                let cmd = &frames[1];
                let svc = &frames[2];
                if cmd == CMD_READY
                    || (cmd == CMD_HEARTBEAT
                        && !is_fed_identity(worker_id)
                        && !self.registry.is_alive(worker_id))
                {
                    if is_fed_identity(worker_id) {
                        if let Some(via) = parse_fed_broker_id(worker_id) {
                            let svc_str = String::from_utf8_lossy(svc).into_owned();
                            learn_peer_broker_id(&mut self.peers, &via);
                            self.registry.register_federated(
                                worker_id.to_vec(),
                                svc_str,
                                now,
                                via.clone(),
                                via.clone(),
                                via,
                            );
                            self.sync(backend)?;
                        }
                    } else {
                        let svc_str = String::from_utf8_lossy(svc).into_owned();
                        self.registry.register(worker_id.to_vec(), svc_str.clone(), now);
                        if let Some(m) = metrics {
                            m.set_workers(&svc_str, self.registry.worker_count(&svc_str) as u64);
                        }
                        self.sync(backend)?;
                    }
                } else if cmd == CMD_HEARTBEAT {
                    self.registry.heartbeat(worker_id, now);
                } else if cmd == CMD_DISCONNECT {
                    let svc_str = String::from_utf8_lossy(svc).into_owned();
                    if is_fed_identity(worker_id) {
                        self.registry.remove_service_binding(worker_id, &svc_str);
                        // Only reclaim when the identity is fully gone.
                        if !self.registry.is_alive(worker_id) {
                            reclaim_worker_inflight(
                                frontend,
                                backend,
                                &self.peers,
                                &mut self.corr,
                                &mut self.in_flight,
                                worker_id,
                                metrics,
                            )?;
                        }
                    } else {
                        let svcs = self.registry.services_of(worker_id);
                        self.registry.remove(worker_id);
                        reclaim_worker_inflight(
                            frontend,
                            backend,
                            &self.peers,
                            &mut self.corr,
                            &mut self.in_flight,
                            worker_id,
                            metrics,
                        )?;
                        if let Some(m) = metrics {
                            for s in svcs {
                                m.set_workers(&s, self.registry.worker_count(&s) as u64);
                            }
                        }
                    }
                    self.sync(backend)?;
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
                if hop_contains(&hop, &self.broker_id) {
                    return Ok(());
                }
                learn_peer_broker_id(&mut self.peers, &via);
                if self.registry.federated_meta(worker_id, &svc)
                    == Some((via.clone(), origin.clone(), hop.clone()))
                {
                    self.registry.heartbeat(worker_id, now);
                } else {
                    self.registry
                        .register_federated(worker_id.to_vec(), svc, now, via, origin, hop);
                    self.sync(backend)?;
                }
                Ok(())
            }
            6 if frames[1] == CMD_FED_CALL => {
                // A broker that joined us calls one of our services.
                let Some(via) = parse_fed_broker_id(worker_id) else {
                    return Ok(());
                };
                let peer_idx = match find_in_link(&self.peers, worker_id) {
                    Some(i) => i,
                    None => self.add_in_link(worker_id, via, None, now),
                };
                self.peers[peer_idx].alive = true;
                self.peers[peer_idx].last_seen = now;
                self.serve_peer_request(
                    peer_idx, frontend, backend, &frames[2], &frames[3], &frames[4], &frames[5],
                    metrics,
                )
            }
            5 => {
                self.registry.release_worker(worker_id);
                let client_id = &frames[1];
                let svc = &frames[2];
                let req_id = &frames[3];
                let body = &frames[4];
                deliver_reply(
                    frontend,
                    backend,
                    &self.peers,
                    &self.client_is_req,
                    &mut self.corr,
                    &mut self.in_flight,
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

    fn add_in_link(
        &mut self,
        identity: &[u8],
        peer_id: String,
        epoch: Option<String>,
        now: Instant,
    ) -> usize {
        self.peers.push(PeerLink {
            kind: LinkKind::In(identity.to_vec()),
            peer_broker_id: Some(peer_id),
            advertised: HashMap::new(),
            join: true,
            passive: false,
            alive: true,
            last_seen: now,
            epoch,
            next_join: now,
        });
        self.peers.len() - 1
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
                log::info!("service federation: broker {via} joined");
                (self.add_in_link(identity, via, Some(epoch), now), true)
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

    /// Periodic work: dead workers, link expiry, advertisement refresh, pending retry.
    fn sweep(
        &mut self,
        frontend: &Socket,
        backend: &Socket,
        now: Instant,
        pending_timeout: Duration,
        metrics: Option<&Arc<ServiceMetrics>>,
    ) -> Result<()> {
        let dead = self.registry.sweep_dead(now, self.timeout);
        for wid in dead {
            reclaim_worker_inflight(
                frontend,
                backend,
                &self.peers,
                &mut self.corr,
                &mut self.in_flight,
                &wid,
                metrics,
            )?;
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
                        "service federation: broker {} stopped answering",
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
                        "service federation: broker {} left",
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
            &self.peers,
            &mut self.pending,
            &mut self.registry,
            &mut self.corr,
            &mut self.in_flight,
            &self.broker_id,
            now,
            pending_timeout,
            self.max_pending,
            metrics,
        )?;
        expire_corr(
            frontend,
            backend,
            &self.peers,
            &mut self.corr,
            now,
            pending_timeout,
            metrics,
        )?;
        Ok(())
    }
}
