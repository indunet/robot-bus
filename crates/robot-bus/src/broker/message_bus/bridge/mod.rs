//! Federated XSUB/XPUB forwarder: broker↔broker topic links.
//!
//! Local clients keep the 2-frame wire format `[topic][payload]`. Between brokers:
//!
//! - **Join links** ([`link`]): the joining broker connects one DEALER to the joined
//!   broker's federation ROUTER. Both sides exchange their subscription demand and send
//!   `DATA` frames only for topics the other side wants. Only the joining broker needs
//!   `--peer`; the joined broker learns about the link when it connects.
//! - **Legacy links** (peer without one-sided join support): a PUB into the peer's XSUB
//!   plus a SUB on the peer's XPUB, 3-frame `[topic][hop_path][payload]` pushes. Both
//!   brokers must configure each other.
//!
//! `hop_path` is a comma-separated list of `broker_id`s (loop prevention). Reserved
//! `/robot_bus` console / status topics stay local: never sent to peers, never accepted
//! from peers, never part of the demand sent to peers.

mod link;

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use crate::zmq::{self, Context as ZmqContext, Socket, SocketType};

use self::link::{
    CMD_DATA, CMD_HELLO, CMD_RESYNC, CMD_SET, CMD_SUB, CMD_UNSUB, CMD_WELCOME, DEFER,
    FULL_SYNC_INTERVAL, HELLO_INTERVAL, LINK_TIMEOUT, Link, Side, topic_wanted,
};
use super::BusConfig;
use super::metrics::MessageMetrics;
use super::peer::MessagePeer;
use crate::console_topics;

const HOP_SEP: char = ',';
const POLL_MS: i64 = 100;

/// Legacy (pre one-sided join) peer: push 3-frame messages, receive via XPUB copies.
struct LegacyLink {
    /// Push federated 3-frame messages into the peer's XSUB.
    pub_sock: Socket,
    /// Express demand on the peer's XPUB; data copies are drained (push is authoritative).
    sub_sock: Socket,
}

/// Where a message entered the forwarder.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// A local client (2-frame message on XSUB).
    Local,
    /// A legacy peer's push (3-frame message on XSUB).
    Legacy,
    /// A join link (index into `Forwarder::links`).
    Link(usize),
}

struct Forwarder {
    id: String,
    epoch: String,
    xsub: Socket,
    xpub: Socket,
    /// Federation ROUTER (joining brokers connect here). `None`: joins disabled.
    router: Option<Socket>,
    metrics: Option<Arc<MessageMetrics>>,
    /// Topic prefixes with a subscriber on our XPUB (local clients and legacy SUB peers).
    client_demand: HashSet<String>,
    /// Topic prefixes currently subscribed on our XSUB (what local publishers see).
    upstream: HashSet<String>,
    legacy: Vec<LegacyLink>,
    links: Vec<Link>,
    /// Peers we configured with a join link; used to decide link ownership.
    configured_join: HashSet<String>,
    demand_dirty: bool,
    snd_hwm: i32,
    rcv_hwm: i32,
}

/// Run the federated forwarder until `shutdown` is set.
pub fn run_federated(
    context: &ZmqContext,
    xsub: Socket,
    xpub: Socket,
    router: Option<Socket>,
    config: &BusConfig,
    shutdown: &AtomicBool,
    metrics: Option<Arc<MessageMetrics>>,
) -> Result<()> {
    let id = if config.broker_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        config.broker_id.clone()
    };

    xpub.set_xpub_verbose(true)
        .context("enable XPUB verbose subscriptions")?;

    let mut fwd = Forwarder {
        id,
        epoch: uuid::Uuid::new_v4().simple().to_string(),
        xsub,
        xpub,
        router,
        metrics,
        client_demand: HashSet::new(),
        upstream: HashSet::new(),
        legacy: Vec::new(),
        links: Vec::new(),
        configured_join: HashSet::new(),
        demand_dirty: false,
        snd_hwm: config.snd_hwm,
        rcv_hwm: config.rcv_hwm,
    };
    let now = Instant::now();
    for peer in &config.peers {
        fwd.add_peer(context, peer, now)?;
    }

    log::info!(
        "message_bus federation forwarder running: broker_id={} peers={} joinable={}",
        fwd.id,
        config.peers.len(),
        fwd.router.is_some()
    );
    fwd.run(shutdown)
}

impl Forwarder {
    fn add_peer(&mut self, context: &ZmqContext, peer: &MessagePeer, now: Instant) -> Result<()> {
        if let Some(fed) = &peer.fed {
            let socket = context
                .socket(SocketType::DEALER)
                .context("create federation DEALER")?;
            socket.set_linger(0).context("federation linger")?;
            socket
                .set_sndhwm(self.snd_hwm.max(1000))
                .context("federation sndhwm")?;
            socket
                .set_rcvhwm(self.rcv_hwm.max(1000))
                .context("federation rcvhwm")?;
            socket
                .set_immediate(true)
                .context("federation immediate")?;
            socket
                .connect(fed)
                .with_context(|| format!("connect federation DEALER -> {fed}"))?;
            if !peer.broker_id.is_empty() {
                self.configured_join.insert(peer.broker_id.clone());
            }
            self.links
                .push(Link::new_out(socket, peer.broker_id.clone(), now));
        } else {
            self.legacy
                .push(connect_legacy(context, peer, self.snd_hwm, self.rcv_hwm)?);
        }
        Ok(())
    }

    fn run(&mut self, shutdown: &AtomicBool) -> Result<()> {
        while !shutdown.load(Ordering::Acquire) {
            let out_links: Vec<usize> = self
                .links
                .iter()
                .enumerate()
                .filter(|(_, l)| matches!(l.side, Side::Out(_)))
                .map(|(i, _)| i)
                .collect();

            let mut items = Vec::with_capacity(3 + self.legacy.len() + out_links.len());
            items.push(self.xsub.as_poll_item(zmq::POLLIN));
            items.push(self.xpub.as_poll_item(zmq::POLLIN));
            let router_slot = self.router.as_ref().map(|r| {
                items.push(r.as_poll_item(zmq::POLLIN));
                items.len() - 1
            });
            let legacy_base = items.len();
            for l in &self.legacy {
                items.push(l.sub_sock.as_poll_item(zmq::POLLIN));
            }
            let out_base = items.len();
            for &i in &out_links {
                if let Side::Out(sock) = &self.links[i].side {
                    items.push(sock.as_poll_item(zmq::POLLIN));
                }
            }

            if zmq::poll(&mut items, POLL_MS).is_err() {
                break;
            }
            let xsub_ready = items[0].is_readable();
            let xpub_ready = items[1].is_readable();
            let router_ready = router_slot.is_some_and(|s| items[s].is_readable());
            let legacy_ready: Vec<bool> = (0..self.legacy.len())
                .map(|i| items[legacy_base + i].is_readable())
                .collect();
            let out_ready: Vec<bool> = (0..out_links.len())
                .map(|i| items[out_base + i].is_readable())
                .collect();
            drop(items);

            if xsub_ready {
                while let Ok(frames) = self.xsub.recv_multipart(zmq::DONTWAIT) {
                    self.on_xsub(&frames)?;
                }
            }
            if xpub_ready {
                while let Ok(frames) = self.xpub.recv_multipart(zmq::DONTWAIT) {
                    self.on_subscription(&frames)?;
                }
            }
            if router_ready {
                self.on_router()?;
            }
            for (i, ready) in legacy_ready.into_iter().enumerate() {
                if ready {
                    drain_peer_sub(&self.legacy[i].sub_sock);
                }
            }
            for (slot, ready) in out_ready.into_iter().enumerate() {
                if ready {
                    self.on_dealer(out_links[slot])?;
                }
            }

            self.tick(Instant::now())?;
        }
        Ok(())
    }

    // ── data path ────────────────────────────────────────────────────────

    fn on_xsub(&mut self, frames: &[Vec<u8>]) -> Result<()> {
        match frames.len() {
            2 => self.route(&frames[0], &frames[1], "", Source::Local),
            3 => {
                let hops = std::str::from_utf8(&frames[1]).unwrap_or("");
                self.route(&frames[0], &frames[2], hops, Source::Legacy)
            }
            // Ignore unexpected shapes (e.g. malformed clients).
            _ => Ok(()),
        }
    }

    /// Deliver a message to local subscribers and forward it to every peer that wants it.
    fn route(&mut self, topic: &[u8], payload: &[u8], hops: &str, src: Source) -> Result<()> {
        let topic_str = std::str::from_utf8(topic).unwrap_or("");
        let federate = should_federate(topic_str);
        if src != Source::Local && (!federate || hop_contains(hops, &self.id)) {
            return Ok(());
        }
        deliver_local(&self.xpub, topic, payload, topic_str, self.metrics.as_ref())?;
        if !federate || (self.legacy.is_empty() && self.links.is_empty()) {
            return Ok(());
        }

        let out_hops = extend_hops(hops, &self.id);
        if !self.legacy.is_empty() && topic_wanted(&self.client_demand, topic_str) {
            push_to_legacy(&self.legacy, topic, out_hops.as_bytes(), payload);
        }
        for (i, link) in self.links.iter().enumerate() {
            if src == Source::Link(i) || !link.active() || hop_contains(hops, &link.peer_id) {
                continue;
            }
            if link.wants(topic_str) {
                link.send(
                    self.router.as_ref(),
                    &[CMD_DATA, topic, out_hops.as_bytes(), payload],
                );
            }
        }
        Ok(())
    }

    // ── demand ───────────────────────────────────────────────────────────

    /// A subscription event from our XPUB (local client or legacy SUB peer).
    fn on_subscription(&mut self, frames: &[Vec<u8>]) -> Result<()> {
        let Some((topic, is_sub)) = parse_subscription(frames) else {
            return Ok(());
        };
        let changed = if is_sub {
            self.client_demand.insert(topic.clone())
        } else {
            self.client_demand.remove(&topic)
        };
        if changed {
            self.refresh_upstream(&topic)?;
            if should_federate(&topic) {
                sync_legacy_subs(&self.legacy, &topic, is_sub)?;
            }
            self.demand_dirty = true;
        }
        Ok(())
    }

    /// Keep our XSUB subscribed to exactly the topics somebody (client or peer) wants,
    /// so local publishers only publish what is needed.
    fn refresh_upstream(&mut self, topic: &str) -> Result<()> {
        let wanted = self.client_demand.contains(topic)
            || self.links.iter().any(|l| l.peer_demand.contains(topic));
        let have = self.upstream.contains(topic);
        if wanted == have {
            return Ok(());
        }
        let mut frame = Vec::with_capacity(topic.len() + 1);
        frame.push(u8::from(wanted));
        frame.extend_from_slice(topic.as_bytes());
        self.xsub
            .send(&frame, 0)
            .context("update XSUB subscription")?;
        if wanted {
            self.upstream.insert(topic.to_string());
        } else {
            self.upstream.remove(topic);
        }
        Ok(())
    }

    /// Replace a link's demand with `new` and update what local publishers see.
    fn set_peer_demand(&mut self, idx: usize, new: HashSet<String>) -> Result<()> {
        let old = std::mem::replace(&mut self.links[idx].peer_demand, new);
        let changed: Vec<String> = old
            .symmetric_difference(&self.links[idx].peer_demand)
            .cloned()
            .collect();
        for topic in &changed {
            self.refresh_upstream(topic)?;
        }
        if !changed.is_empty() {
            self.demand_dirty = true;
        }
        Ok(())
    }

    /// What we want from link `idx`: our clients' demand plus every other link's demand
    /// (so brokers in a chain relay), minus reserved topics.
    fn desired_demand(&self, idx: usize) -> HashSet<String> {
        let mut out: HashSet<String> = self
            .client_demand
            .iter()
            .filter(|t| should_federate(t))
            .cloned()
            .collect();
        for (i, other) in self.links.iter().enumerate() {
            if i != idx && other.active() {
                out.extend(other.peer_demand.iter().filter(|t| should_federate(t)).cloned());
            }
        }
        out
    }

    fn sync_demand(&mut self, now: Instant) {
        for idx in 0..self.links.len() {
            if !self.links[idx].active() {
                continue;
            }
            let desired = self.desired_demand(idx);
            let router = self.router.as_ref();
            let link = &mut self.links[idx];
            if now >= link.full_due {
                let mut frames: Vec<&[u8]> = Vec::with_capacity(desired.len() + 1);
                frames.push(CMD_SET);
                frames.extend(desired.iter().map(|t| t.as_bytes()));
                if link.send(router, &frames) {
                    link.sent_demand = desired;
                    link.full_due = now + FULL_SYNC_INTERVAL;
                }
                continue;
            }
            let added: Vec<String> = desired.difference(&link.sent_demand).cloned().collect();
            let removed: Vec<String> = link.sent_demand.difference(&desired).cloned().collect();
            for topic in added {
                if link.send(router, &[CMD_SUB, topic.as_bytes()]) {
                    link.sent_demand.insert(topic);
                }
            }
            for topic in removed {
                if link.send(router, &[CMD_UNSUB, topic.as_bytes()]) {
                    link.sent_demand.remove(&topic);
                }
            }
        }
    }

    // ── link control ─────────────────────────────────────────────────────

    fn on_router(&mut self) -> Result<()> {
        loop {
            let Some(router) = &self.router else {
                return Ok(());
            };
            let Ok(frames) = router.recv_multipart(zmq::DONTWAIT) else {
                return Ok(());
            };
            if frames.len() < 2 {
                continue;
            }
            let now = Instant::now();
            if frames[1] == CMD_HELLO {
                self.on_hello(&frames, now)?;
                continue;
            }
            let Some(idx) = self.links.iter().position(|l| l.is_in(&frames[0])) else {
                continue; // not joined (or expired): its next HELLO re-creates the link
            };
            self.links[idx].last_seen = now;
            self.on_link_frame(idx, &frames[1], &frames[2..], now)?;
        }
    }

    /// `[identity] HELLO id epoch`
    fn on_hello(&mut self, frames: &[Vec<u8>], now: Instant) -> Result<()> {
        if frames.len() < 4 {
            return Ok(());
        }
        let identity = frames[0].clone();
        let peer_id = String::from_utf8_lossy(&frames[2]).into_owned();
        let epoch = String::from_utf8_lossy(&frames[3]).into_owned();
        let existing = self.links.iter().position(|l| l.is_in(&identity));

        // Both brokers configured each other: the link belongs to the smaller id. We keep
        // ours and tell the joiner to stay passive.
        if self.id.as_str() < peer_id.as_str() && self.configured_join.contains(&peer_id) {
            if let Some(idx) = existing {
                self.remove_link(idx)?;
            }
            self.send_router(
                &identity,
                &[CMD_WELCOME, self.id.as_bytes(), self.epoch.as_bytes(), DEFER],
            );
            return Ok(());
        }

        if let Some(idx) = existing {
            if self.links[idx].peer_epoch.as_deref() == Some(epoch.as_str()) {
                self.links[idx].last_seen = now;
                self.send_router(
                    &identity,
                    &[CMD_WELCOME, self.id.as_bytes(), self.epoch.as_bytes()],
                );
                return Ok(());
            }
            // Same connection, new process state: forget what the old one wanted.
            self.remove_link(idx)?;
        }

        log::info!("federation: broker {peer_id} joined");
        self.links
            .push(Link::new_in(identity.clone(), peer_id, epoch, now));
        self.send_router(
            &identity,
            &[CMD_WELCOME, self.id.as_bytes(), self.epoch.as_bytes()],
        );
        // We know nothing about what it wants; it knows nothing about us.
        self.send_router(&identity, &[CMD_RESYNC]);
        self.demand_dirty = true;
        Ok(())
    }

    fn send_router(&self, identity: &[u8], frames: &[&[u8]]) {
        if let Some(router) = &self.router {
            let _ = router.send_multipart(
                std::iter::once(identity).chain(frames.iter().copied()),
                zmq::DONTWAIT,
            );
        }
    }

    fn on_dealer(&mut self, idx: usize) -> Result<()> {
        loop {
            let frames = {
                let Side::Out(sock) = &self.links[idx].side else {
                    return Ok(());
                };
                match sock.recv_multipart(zmq::DONTWAIT) {
                    Ok(f) => f,
                    Err(_) => return Ok(()),
                }
            };
            if frames.is_empty() {
                continue;
            }
            let now = Instant::now();
            self.links[idx].last_seen = now;
            if frames[0] == CMD_WELCOME {
                self.on_welcome(idx, &frames, now)?;
            } else {
                self.on_link_frame(idx, &frames[0], &frames[1..], now)?;
            }
        }
    }

    /// `WELCOME id epoch [DEFER]`
    fn on_welcome(&mut self, idx: usize, frames: &[Vec<u8>], now: Instant) -> Result<()> {
        if frames.len() < 3 {
            return Ok(());
        }
        let peer_id = String::from_utf8_lossy(&frames[1]).into_owned();
        let epoch = String::from_utf8_lossy(&frames[2]).into_owned();
        let defer = frames.get(3).is_some_and(|f| f == DEFER);

        let (mut resync, mut forget_peer) = (false, false);
        {
            let link = &mut self.links[idx];
            if link.peer_id.is_empty() {
                link.peer_id = peer_id.clone();
            }
            if !link.alive {
                link.alive = true;
                resync = true;
                log::info!("federation: link to broker {peer_id} is up");
            }
            if link.peer_epoch.as_deref().is_some_and(|old| old != epoch) {
                // The peer restarted: what it wanted before is gone, and it knows nothing of us.
                forget_peer = true;
                resync = true;
            }
            link.peer_epoch = Some(epoch);
            if link.passive != defer {
                link.passive = defer;
                if defer {
                    log::info!(
                        "federation: broker {peer_id} also configured this broker; its link \
                         carries the traffic, keeping this connection idle"
                    );
                    forget_peer = true;
                    link.sent_demand.clear();
                } else {
                    log::info!("federation: link to broker {peer_id} is active again");
                    resync = true;
                }
            }
        }
        if forget_peer {
            self.set_peer_demand(idx, HashSet::new())?;
        }
        if resync && !self.links[idx].passive {
            let link = &mut self.links[idx];
            link.sent_demand.clear();
            link.full_due = now;
            link.send(self.router.as_ref(), &[CMD_RESYNC]);
            self.demand_dirty = true;
        }
        Ok(())
    }

    /// Frames shared by both link sides (`SUB`, `UNSUB`, `SET`, `RESYNC`, `DATA`).
    fn on_link_frame(&mut self, idx: usize, cmd: &[u8], rest: &[Vec<u8>], now: Instant) -> Result<()> {
        if self.links[idx].passive {
            return Ok(());
        }
        if cmd == CMD_DATA {
            if rest.len() == 3 {
                let hops = std::str::from_utf8(&rest[1]).unwrap_or("");
                self.route(&rest[0], &rest[2], hops, Source::Link(idx))?;
            }
        } else if cmd == CMD_SUB || cmd == CMD_UNSUB {
            let Some(topic) = rest.first() else {
                return Ok(());
            };
            let topic = String::from_utf8_lossy(topic).into_owned();
            if !should_federate(&topic) {
                return Ok(());
            }
            let changed = if cmd == CMD_SUB {
                self.links[idx].peer_demand.insert(topic.clone())
            } else {
                self.links[idx].peer_demand.remove(&topic)
            };
            if changed {
                self.refresh_upstream(&topic)?;
                self.demand_dirty = true;
            }
        } else if cmd == CMD_SET {
            let new: HashSet<String> = rest
                .iter()
                .map(|t| String::from_utf8_lossy(t).into_owned())
                .filter(|t| should_federate(t))
                .collect();
            self.set_peer_demand(idx, new)?;
        } else if cmd == CMD_RESYNC {
            self.links[idx].full_due = now;
            self.demand_dirty = true;
        }
        Ok(())
    }

    fn remove_link(&mut self, idx: usize) -> Result<()> {
        self.set_peer_demand(idx, HashSet::new())?;
        let link = self.links.remove(idx);
        log::info!("federation: link to broker {} closed", link.peer_id);
        Ok(())
    }

    /// Timers: hello, link expiry, demand sync.
    fn tick(&mut self, now: Instant) -> Result<()> {
        let mut idx = 0;
        while idx < self.links.len() {
            match &self.links[idx].side {
                Side::Out(_) => {
                    if now >= self.links[idx].next_hello {
                        self.links[idx].next_hello = now + HELLO_INTERVAL;
                        let (id, epoch) = (self.id.clone(), self.epoch.clone());
                        self.links[idx].send(None, &[CMD_HELLO, id.as_bytes(), epoch.as_bytes()]);
                    }
                    let link = &self.links[idx];
                    if link.alive && now.duration_since(link.last_seen) > LINK_TIMEOUT {
                        log::warn!("federation: link to broker {} is down", link.peer_id);
                        self.set_peer_demand(idx, HashSet::new())?;
                        let link = &mut self.links[idx];
                        link.alive = false;
                        link.peer_epoch = None;
                        link.sent_demand.clear();
                    }
                    idx += 1;
                }
                Side::In(_) => {
                    if now.duration_since(self.links[idx].last_seen) > LINK_TIMEOUT {
                        self.remove_link(idx)?;
                    } else {
                        idx += 1;
                    }
                }
            }
        }

        let sync_due = self
            .links
            .iter()
            .any(|l| l.active() && now >= l.full_due);
        if self.demand_dirty || sync_due {
            self.demand_dirty = false;
            self.sync_demand(now);
        }
        Ok(())
    }
}

// ── legacy links ─────────────────────────────────────────────────────────

fn connect_legacy(
    context: &ZmqContext,
    peer: &MessagePeer,
    snd_hwm: i32,
    rcv_hwm: i32,
) -> Result<LegacyLink> {
    log::warn!(
        "federation: peer {} does not support one-sided join; it only sees this broker's \
         topics if --peer is configured on both brokers",
        if peer.broker_id.is_empty() {
            peer.xpub.as_str()
        } else {
            peer.broker_id.as_str()
        }
    );
    let pub_sock = context
        .socket(SocketType::PUB)
        .context("create federation PUB")?;
    pub_sock.set_linger(0).context("peer PUB linger")?;
    pub_sock.set_sndhwm(snd_hwm).context("peer PUB sndhwm")?;
    pub_sock.set_immediate(true).context("peer PUB immediate")?;
    pub_sock
        .connect(&peer.xsub)
        .with_context(|| format!("connect federation PUB -> {}", peer.xsub))?;

    let sub_sock = context
        .socket(SocketType::SUB)
        .context("create federation SUB")?;
    sub_sock.set_linger(0).context("peer SUB linger")?;
    sub_sock
        .set_rcvhwm(rcv_hwm.max(100))
        .context("peer SUB rcvhwm")?;
    sub_sock
        .connect(&peer.xpub)
        .with_context(|| format!("connect federation SUB -> {}", peer.xpub))?;

    Ok(LegacyLink { pub_sock, sub_sock })
}

fn push_to_legacy(peers: &[LegacyLink], topic: &[u8], hops: &[u8], payload: &[u8]) {
    for link in peers {
        // DONTWAIT: slow / disconnected peers must not stall the local bus.
        let _ = link
            .pub_sock
            .send_multipart([topic, hops, payload], zmq::DONTWAIT);
    }
}

fn sync_legacy_subs(peers: &[LegacyLink], topic: &str, subscribe: bool) -> Result<()> {
    for link in peers {
        if subscribe {
            link.sub_sock
                .set_subscribe(topic.as_bytes())
                .with_context(|| format!("peer SUB subscribe {topic}"))?;
        } else {
            link.sub_sock
                .set_unsubscribe(topic.as_bytes())
                .with_context(|| format!("peer SUB unsubscribe {topic}"))?;
        }
    }
    Ok(())
}

fn drain_peer_sub(sub: &Socket) {
    while sub.recv_multipart(zmq::DONTWAIT).is_ok() {}
}

// ── helpers ──────────────────────────────────────────────────────────────

fn deliver_local(
    xpub: &Socket,
    topic: &[u8],
    payload: &[u8],
    topic_str: &str,
    metrics: Option<&Arc<MessageMetrics>>,
) -> Result<()> {
    xpub.send_multipart([topic, payload], 0)
        .context("deliver to local XPUB")?;
    if let Some(m) = metrics {
        let bytes = (topic.len() + payload.len()) as u64;
        m.record(topic_str, bytes);
    }
    Ok(())
}

fn parse_subscription(frames: &[Vec<u8>]) -> Option<(String, bool)> {
    let frame = frames.first()?;
    if frame.is_empty() {
        return None;
    }
    let is_sub = match frame[0] {
        1 => true,
        0 => false,
        _ => return None,
    };
    let topic = String::from_utf8_lossy(&frame[1..]).into_owned();
    Some((topic, is_sub))
}

/// Console / status namespace stays per-broker; do not federate `/robot_bus*`.
fn should_federate(topic: &str) -> bool {
    let logical = crate::runtime::retained_topic::internal_source_topic(topic).unwrap_or(topic);
    !console_topics::is_reserved_name(logical)
}

fn hop_contains(hops: &str, broker_id: &str) -> bool {
    !broker_id.is_empty() && hops.split(HOP_SEP).any(|h| h == broker_id)
}

fn extend_hops(hops: &str, broker_id: &str) -> String {
    if hops.is_empty() {
        broker_id.to_string()
    } else {
        format!("{hops}{HOP_SEP}{broker_id}")
    }
}

#[cfg(test)]
mod hop_tests {
    use super::*;

    #[test]
    fn hop_path_roundtrip() {
        assert!(!hop_contains("a,b", "c"));
        assert!(hop_contains("a,b", "b"));
        assert!(!hop_contains("a,b", ""));
        assert_eq!(extend_hops("a", "b"), "a,b");
        assert_eq!(extend_hops("", "a"), "a");
    }

    #[test]
    fn topic_prefix_match() {
        let mut s = HashSet::new();
        assert!(!topic_wanted(&s, "robot/pose"));
        s.insert("robot/".to_string());
        assert!(topic_wanted(&s, "robot/pose"));
        assert!(!topic_wanted(&s, "other"));
        s.insert(String::new());
        assert!(topic_wanted(&s, "other"));
    }

    #[test]
    fn reserved_console_topics_are_not_federated() {
        assert!(!should_federate("/robot_bus/status"));
        assert!(!should_federate("/robot_bus/topics"));
        assert!(!should_federate("/robot_bus/topology"));
        assert!(!should_federate("/robot_bus/tank/pose"));
        assert!(!should_federate("/robot_bus"));
        assert!(should_federate("fleet/pose"));
        assert!(should_federate("/robot1/imu"));
    }
}
