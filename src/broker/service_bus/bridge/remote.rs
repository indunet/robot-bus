//! Peer DEALER links and federation correlation helpers.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;
use zmq::{Context as ZmqContext, Socket, SocketType};

use super::protocol::{FED_ID_PREFIX, FED_REQ_PREFIX};

pub(super) struct PeerLink {
    pub(super) dealer: Socket,
    /// Peer broker id (from config or learned from reverse READY_FED).
    pub(super) peer_broker_id: Option<String>,
    /// service -> (origin, hop) we currently advertise on this link.
    pub(super) advertised: HashMap<String, (String, String)>,
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

pub(super) fn parse_fed_broker_id(identity: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(identity).ok()?;
    s.strip_prefix(FED_ID_PREFIX)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

pub(super) fn is_fed_identity(identity: &[u8]) -> bool {
    parse_fed_broker_id(identity).is_some()
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

/// Fill in a peer link's broker id when learned from a reverse registration.
pub(super) fn learn_peer_broker_id(peers: &mut [PeerLink], via: &str) {
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
