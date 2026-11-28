//! Peer DEALER links and federation correlation helpers.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;
use crate::zmq::{self, Context as ZmqContext, Socket, SocketType};

use super::protocol::{CMD_FED_REPLY, FED_ID_PREFIX, FED_REQ_PREFIX, LINK_ID_PREFIX};

/// How a link reaches the peer.
pub(super) enum LinkKind {
    /// Our DEALER connected to the peer's service backend (a configured `--peer`).
    Out(Socket),
    /// The peer joined our backend (`fed/<id>` identity); we answer through the backend ROUTER.
    In(Vec<u8>),
}

pub(super) struct PeerLink {
    pub(super) kind: LinkKind,
    /// Peer broker id (from config, a `JOIN`, or learned from reverse READY_FED).
    pub(super) peer_broker_id: Option<String>,
    /// service -> (origin, hop) we currently advertise on this link.
    pub(super) advertised: HashMap<String, (String, String)>,
    /// Out: the peer supports one-sided join (we send `JOIN` and use its services).
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

    pub(super) fn is_in(&self, identity: &[u8]) -> bool {
        matches!(&self.kind, LinkKind::In(id) if id == identity)
    }

    /// Our DEALER (Out links only).
    pub(super) fn dealer(&self) -> Option<&Socket> {
        match &self.kind {
            LinkKind::Out(sock) => Some(sock),
            LinkKind::In(_) => None,
        }
    }

    /// Send control / reply frames toward the peer without blocking.
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

    /// Send the answer to a request that came in over this link.
    ///
    /// Out (the peer calls us as a worker): the plain 4-frame reply the peer's backend
    /// expects. In (the peer joined us): a `FED_REPLY`.
    pub(super) fn send_reply(
        &self,
        backend: &Socket,
        client_id: &[u8],
        svc: &[u8],
        req_id: &[u8],
        body: &[u8],
    ) -> bool {
        match &self.kind {
            LinkKind::Out(_) => self.send(backend, &[client_id, svc, req_id, body]),
            LinkKind::In(_) => self.send(backend, &[CMD_FED_REPLY, client_id, svc, req_id, body]),
        }
    }

    /// Registry identity of the services this peer offers over our own joining link.
    pub(super) fn link_identity(&self) -> Option<Vec<u8>> {
        match (&self.kind, &self.peer_broker_id) {
            (LinkKind::Out(_), Some(id)) if self.join => Some(link_identity(id)),
            _ => None,
        }
    }
}

pub(super) fn link_identity(peer_id: &str) -> Vec<u8> {
    format!("{LINK_ID_PREFIX}{peer_id}").into_bytes()
}

pub(super) enum ReplyTarget {
    Frontend { has_req_delim: bool },
    Peer { peer_idx: usize },
}

pub(super) struct CorrEntry {
    pub(super) target: ReplyTarget,
    pub(super) original_client_id: Vec<u8>,
    pub(super) service: Vec<u8>,
    pub(super) request_id: Vec<u8>,
    pub(super) worker_identity: Vec<u8>,
    pub(super) queued_at: Instant,
}

pub(super) struct PendingRequest {
    pub(super) client_identity: Vec<u8>,
    pub(super) service: Vec<u8>,
    pub(super) request_id: Vec<u8>,
    pub(super) body: Vec<u8>,
    pub(super) has_req_delim: bool,
    pub(super) hop_path: String,
    pub(super) peer_idx: Option<usize>,
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
        .context("create federation DEALER")?;
    dealer.set_linger(0).context("set linger")?;
    // Calls and control frames share this pipe and are sent without blocking.
    dealer.set_sndhwm(snd_hwm.max(1000)).context("set sndhwm")?;
    dealer.set_rcvhwm(rcv_hwm.max(1000)).context("set rcvhwm")?;
    dealer.set_immediate(true).context("set immediate")?;
    let identity = format!("{FED_ID_PREFIX}{broker_id}");
    dealer
        .set_identity(identity.as_bytes())
        .context("set federation identity")?;
    dealer
        .connect(backend)
        .with_context(|| format!("connect federation DEALER to {backend}"))?;
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
        last_seen: Instant::now(),
        epoch: None,
        next_join: Instant::now(),
    })
}

pub(super) fn parse_fed_broker_id(identity: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(identity).ok()?;
    s.strip_prefix(FED_ID_PREFIX)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

/// Registry identity of a peer route: a worker-side `fed/<id>` DEALER or our own
/// `fedlink/<id>` joining link. Requests to either need a correlation id.
pub(in crate::broker::service_bus) fn is_fed_identity(identity: &[u8]) -> bool {
    parse_fed_broker_id(identity).is_some() || parse_link_broker_id(identity).is_some()
}

pub(super) fn parse_link_broker_id(identity: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(identity).ok()?;
    s.strip_prefix(LINK_ID_PREFIX)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

pub(super) fn make_corr_id(hops: &str) -> Vec<u8> {
    let corr = Uuid::new_v4().simple().to_string();
    format!("{FED_REQ_PREFIX}{corr}/{hops}").into_bytes()
}

pub(super) fn parse_corr_id(id: &[u8]) -> Option<(String, String)> {
    let s = std::str::from_utf8(id).ok()?;
    let rest = s.strip_prefix(FED_REQ_PREFIX)?;
    let (corr, hops) = rest.split_once('/')?;
    Some((corr.to_string(), hops.to_string()))
}

/// Fill in an outgoing link's broker id when learned from a reverse registration.
pub(super) fn learn_peer_broker_id(peers: &mut [PeerLink], via: &str) {
    if peers
        .iter()
        .any(|p| p.peer_broker_id.as_deref() == Some(via))
    {
        return;
    }
    if let Some(link) = peers
        .iter_mut()
        .find(|p| p.peer_broker_id.is_none() && p.dealer().is_some())
    {
        link.peer_broker_id = Some(via.to_string());
    }
}

/// Index of the outgoing joining link to `peer_id`.
pub(super) fn find_join_link(peers: &[PeerLink], peer_id: &str) -> Option<usize> {
    peers.iter().position(|p| {
        p.join && p.dealer().is_some() && p.peer_broker_id.as_deref() == Some(peer_id)
    })
}

/// Index of the joined-us link with this backend identity.
pub(super) fn find_in_link(peers: &[PeerLink], identity: &[u8]) -> Option<usize> {
    peers.iter().position(|p| p.is_in(identity))
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
