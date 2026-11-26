//! Peer DEALER links and the RemoteActions routing table.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use zmq::{Context as ZmqContext, Socket, SocketType};

use super::super::broker::{GoalReply, extend_hops};
use super::protocol::FED_ID_PREFIX;

/// How a link reaches the peer.
pub(super) enum LinkKind {
    /// Our DEALER connected to the peer's action backend (a configured `--peer`).
    Out(Socket),
    /// The peer joined our backend (`fed/<id>` identity); we talk to it through the ROUTER.
    In(Vec<u8>),
}

pub(super) struct PeerLink {
    pub(super) kind: LinkKind,
    /// Peer broker id (from config, a `JOIN`, or learned from reverse READY_FED).
    pub(super) peer_broker_id: Option<String>,
    /// action -> (origin, hop) currently advertised on this link.
    pub(super) advertised: HashMap<String, (String, String)>,
    /// Out: the peer supports one-sided join (we send `JOIN` and use its actions).
    pub(super) join: bool,
    /// Out + join: the peer owns this pair's link; we only keep the connection.
    pub(super) passive: bool,
    /// Out + join: the peer answered `JOIN` recently. In: the peer sent `JOIN` recently.
    pub(super) alive: bool,
    pub(super) last_seen: Instant,
    /// The peer's process epoch (from `JOIN` / `JOIN_ACK`); a change means it restarted.
    pub(super) epoch: Option<String>,
    pub(super) next_join: Instant,
}

impl PeerLink {
    /// Advertisements and heartbeats flow on this link.
    pub(super) fn enabled(&self) -> bool {
        match &self.kind {
            // A joining link speaks only after the peer acknowledged our `JOIN`, so the peer
            // has a link for us before the first advertisement arrives.
            LinkKind::Out(_) => !self.join || (self.alive && !self.passive),
            LinkKind::In(_) => self.alive,
        }
    }

    pub(super) fn dealer(&self) -> Option<&Socket> {
        match &self.kind {
            LinkKind::Out(sock) => Some(sock),
            LinkKind::In(_) => None,
        }
    }

    /// Send frames toward the peer without blocking.
    pub(super) fn send(&self, backend: &Socket, frames: &[&[u8]]) -> bool {
        match &self.kind {
            LinkKind::Out(dealer) => dealer
                .send_multipart(frames.iter().copied(), zmq::DONTWAIT)
                .is_ok(),
            LinkKind::In(identity) => backend
                .send_multipart(
                    std::iter::once(identity.as_slice()).chain(frames.iter().copied()),
                    zmq::DONTWAIT,
                )
                .is_ok(),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct RemoteRoute {
    pub(super) peer_idx: usize,
    pub(super) origin_broker_id: String,
    pub(super) hop_path: String,
    pub(super) last_heartbeat: Instant,
}

/// action_name -> remote routes.
pub(super) struct RemoteActions {
    pub(super) by_action: HashMap<String, Vec<RemoteRoute>>,
}

impl RemoteActions {
    pub(super) fn new() -> Self {
        Self {
            by_action: HashMap::new(),
        }
    }

    pub(super) fn upsert(
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

    /// Refresh heartbeats for `peer_idx`/`action`. Returns `true` if a route
    /// already existed (and was refreshed); `false` if the peer has no route
    /// for this action yet (caller may bootstrap from a direct HEARTBEAT).
    pub(super) fn heartbeat_peer(&mut self, peer_idx: usize, action: &str, now: Instant) -> bool {
        if let Some(list) = self.by_action.get_mut(action) {
            let mut found = false;
            for r in list.iter_mut().filter(|r| r.peer_idx == peer_idx) {
                r.last_heartbeat = now;
                found = true;
            }
            return found;
        }
        false
    }

    pub(super) fn remove_action_peer(&mut self, peer_idx: usize, action: &str) {
        if let Some(list) = self.by_action.get_mut(action) {
            list.retain(|r| r.peer_idx != peer_idx);
            if list.is_empty() {
                self.by_action.remove(action);
            }
        }
    }

    /// Drop every route learned over `peer_idx`. Returns `true` if any existed.
    pub(super) fn remove_peer(&mut self, peer_idx: usize) -> bool {
        let mut any = false;
        self.by_action.retain(|_, list| {
            let before = list.len();
            list.retain(|r| r.peer_idx != peer_idx);
            any |= list.len() != before;
            !list.is_empty()
        });
        any
    }

    /// `true` if `peer_idx` already has exactly this route for `action`.
    pub(super) fn has_route(&self, action: &str, peer_idx: usize, origin: &str, hop: &str) -> bool {
        self.by_action.get(action).is_some_and(|l| {
            l.iter().any(|r| {
                r.peer_idx == peer_idx && r.origin_broker_id == origin && r.hop_path == hop
            })
        })
    }

    pub(super) fn sweep_dead(&mut self, now: Instant, timeout: Duration) -> Vec<usize> {
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
    pub(super) fn advertise_snapshot(&self, self_id: &str) -> Vec<(String, String, String, usize)> {
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

pub(super) struct PendingGoal {
    pub(super) client_identity: Vec<u8>,
    pub(super) action: Vec<u8>,
    pub(super) goal_id: Vec<u8>,
    pub(super) body: Vec<u8>,
    pub(super) hop_path: String,
    pub(super) reply: GoalReply,
    pub(super) queued_at: Instant,
}

pub(super) fn connect_peer(
    context: &ZmqContext,
    backend: &str,
    broker_id: &str,
    peer_broker_id: &str,
    join: bool,
    snd_hwm: i32,
    rcv_hwm: i32,
) -> Result<PeerLink> {
    let dealer = context
        .socket(SocketType::DEALER)
        .context("create action federation DEALER")?;
    dealer.set_linger(0).context("set linger")?;
    // Goals and control frames share this pipe and are sent without blocking.
    dealer.set_sndhwm(snd_hwm.max(1000)).context("set sndhwm")?;
    dealer.set_rcvhwm(rcv_hwm.max(1000)).context("set rcvhwm")?;
    dealer.set_immediate(true).context("set immediate")?;
    let identity = format!("{FED_ID_PREFIX}{broker_id}");
    dealer
        .set_identity(identity.as_bytes())
        .context("set federation identity")?;
    dealer
        .connect(backend)
        .with_context(|| format!("connect action federation DEALER to {backend}"))?;
    let now = Instant::now();
    Ok(PeerLink {
        kind: LinkKind::Out(dealer),
        peer_broker_id: if peer_broker_id.is_empty() {
            None
        } else {
            Some(peer_broker_id.to_string())
        },
        advertised: HashMap::new(),
        join,
        passive: false,
        alive: false,
        last_seen: now,
        epoch: None,
        next_join: now,
    })
}

pub(super) fn parse_fed_broker_id(identity: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(identity).ok()?;
    s.strip_prefix(FED_ID_PREFIX)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

pub(in crate::broker::action_bus) fn is_fed_identity(identity: &[u8]) -> bool {
    parse_fed_broker_id(identity).is_some()
}

/// The link a message from `fed/<via>` on our backend belongs to: the link that peer joined
/// us through, else a configured link (mutual legacy configuration).
pub(super) fn learn_peer_broker_id(peers: &mut [PeerLink], via: &str) -> Option<usize> {
    if let Some(idx) = peers.iter().position(|p| {
        matches!(p.kind, LinkKind::In(_)) && p.peer_broker_id.as_deref() == Some(via)
    }) {
        return Some(idx);
    }
    if let Some(idx) = peers.iter().position(|p| {
        matches!(p.kind, LinkKind::Out(_)) && p.peer_broker_id.as_deref() == Some(via)
    }) {
        return Some(idx);
    }
    if let Some((idx, link)) = peers
        .iter_mut()
        .enumerate()
        .find(|(_, p)| matches!(p.kind, LinkKind::Out(_)) && p.peer_broker_id.is_none())
    {
        link.peer_broker_id = Some(via.to_string());
        return Some(idx);
    }
    None
}

pub(super) fn find_in_link(peers: &[PeerLink], identity: &[u8]) -> Option<usize> {
    peers
        .iter()
        .position(|p| matches!(&p.kind, LinkKind::In(id) if id == identity))
}

/// Our own joining link to `peer_id`.
pub(super) fn find_join_link(peers: &[PeerLink], peer_id: &str) -> Option<usize> {
    peers.iter().position(|p| {
        matches!(p.kind, LinkKind::Out(_)) && p.join && p.peer_broker_id.as_deref() == Some(peer_id)
    })
}
