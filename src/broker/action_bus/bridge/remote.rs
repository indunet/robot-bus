//! Peer DEALER links and the RemoteActions routing table.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use zmq::{Context as ZmqContext, Socket, SocketType};

use super::protocol::FED_ID_PREFIX;
use super::super::broker::{extend_hops, GoalReply};

pub(super) struct PeerLink {
    pub(super) dealer: Socket,
    pub(super) peer_broker_id: Option<String>,
    /// action -> (origin, hop) currently advertised on this link.
    pub(super) advertised: HashMap<String, (String, String)>,
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

    pub(super) fn heartbeat_peer(&mut self, peer_idx: usize, action: &str, now: Instant) {
        if let Some(list) = self.by_action.get_mut(action) {
            for r in list.iter_mut().filter(|r| r.peer_idx == peer_idx) {
                r.last_heartbeat = now;
            }
        }
    }

    pub(super) fn remove_action_peer(&mut self, peer_idx: usize, action: &str) {
        if let Some(list) = self.by_action.get_mut(action) {
            list.retain(|r| r.peer_idx != peer_idx);
            if list.is_empty() {
                self.by_action.remove(action);
            }
        }
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

pub(super) fn parse_fed_broker_id(identity: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(identity).ok()?;
    s.strip_prefix(FED_ID_PREFIX)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

pub(super) fn is_fed_identity(identity: &[u8]) -> bool {
    parse_fed_broker_id(identity).is_some()
}

pub(super) fn find_peer_idx(peers: &[PeerLink], broker_id: &str) -> Option<usize> {
    peers
        .iter()
        .position(|p| p.peer_broker_id.as_deref() == Some(broker_id))
}

pub(super) fn learn_peer_broker_id(peers: &mut [PeerLink], via: &str) -> Option<usize> {
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
