//! Federation links over the dedicated federation channel.
//!
//! The joining broker (A) connects a DEALER to the joined broker's (B) federation
//! ROUTER. Everything runs over that one connection, initiated by A:
//!
//! | frames (DEALER view)                  | meaning                                      |
//! |---------------------------------------|----------------------------------------------|
//! | `HELLO  id epoch`            A → B    | keep-alive / join request (every second)     |
//! | `WELCOME id epoch [DEFER]`   B → A    | answer to HELLO; `DEFER`: B owns the link    |
//! | `SUB topic` / `UNSUB topic`  both     | incremental change of the sender's demand    |
//! | `SET topic…`                 both     | the sender's whole demand (periodic sync)    |
//! | `RESYNC`                     both     | "I lost your demand: send `SET` again"       |
//! | `DATA topic hops payload`    both     | a message the receiver's side wants          |
//!
//! "Demand" is the set of topic prefixes the sender's side has subscribers for. A
//! broker only sends a topic over a link when the other side asked for it.

use std::collections::HashSet;
use std::time::{Duration, Instant};
use zmq::Socket;

pub(super) const CMD_HELLO: &[u8] = b"HELLO";
pub(super) const CMD_WELCOME: &[u8] = b"WELCOME";
pub(super) const CMD_SUB: &[u8] = b"SUB";
pub(super) const CMD_UNSUB: &[u8] = b"UNSUB";
pub(super) const CMD_SET: &[u8] = b"SET";
pub(super) const CMD_RESYNC: &[u8] = b"RESYNC";
pub(super) const CMD_DATA: &[u8] = b"DATA";
/// Fourth `WELCOME` frame: both brokers configured each other and the answering broker
/// owns the link (its id sorts first); the joining side keeps the connection only.
pub(super) const DEFER: &[u8] = b"DEFER";

pub(super) const HELLO_INTERVAL: Duration = Duration::from_secs(1);
/// A link without any frame from the peer for this long is considered gone.
pub(super) const LINK_TIMEOUT: Duration = Duration::from_secs(5);
/// Re-send the whole demand this often (heals lost `SUB` / `UNSUB` frames).
pub(super) const FULL_SYNC_INTERVAL: Duration = Duration::from_secs(10);

pub(super) enum Side {
    /// We connected (a configured `--peer`).
    Out(Socket),
    /// The peer joined us; its ROUTER identity.
    In(Vec<u8>),
}

pub(super) struct Link {
    pub(super) side: Side,
    pub(super) peer_id: String,
    /// Topic prefixes the peer wants from us.
    pub(super) peer_demand: HashSet<String>,
    /// What we last told the peer we want from it.
    pub(super) sent_demand: HashSet<String>,
    /// The peer answered recently.
    pub(super) alive: bool,
    /// Out links only: the peer owns this pair's link; we keep the connection but
    /// exchange no demand or data.
    pub(super) passive: bool,
    pub(super) last_seen: Instant,
    pub(super) next_hello: Instant,
    /// Out: epoch from the peer's last `WELCOME`; In: epoch from its `HELLO`.
    pub(super) peer_epoch: Option<String>,
    /// Send a full `SET` once this time has passed.
    pub(super) full_due: Instant,
}

impl Link {
    pub(super) fn new_out(socket: Socket, peer_id: String, now: Instant) -> Self {
        Self {
            side: Side::Out(socket),
            peer_id,
            peer_demand: HashSet::new(),
            sent_demand: HashSet::new(),
            alive: false,
            passive: false,
            last_seen: now,
            next_hello: now,
            peer_epoch: None,
            full_due: now,
        }
    }

    pub(super) fn new_in(identity: Vec<u8>, peer_id: String, epoch: String, now: Instant) -> Self {
        Self {
            side: Side::In(identity),
            peer_id,
            peer_demand: HashSet::new(),
            sent_demand: HashSet::new(),
            alive: true,
            passive: false,
            last_seen: now,
            next_hello: now,
            peer_epoch: Some(epoch),
            full_due: now,
        }
    }

    /// Demand and data may flow.
    pub(super) fn active(&self) -> bool {
        self.alive && !self.passive
    }

    pub(super) fn wants(&self, topic: &str) -> bool {
        topic_wanted(&self.peer_demand, topic)
    }

    pub(super) fn is_in(&self, identity: &[u8]) -> bool {
        matches!(&self.side, Side::In(id) if id == identity)
    }

    /// Send without blocking. Returns whether the frames were accepted.
    pub(super) fn send(&self, router: Option<&Socket>, frames: &[&[u8]]) -> bool {
        match &self.side {
            Side::Out(dealer) => dealer
                .send_multipart(frames.iter().copied(), zmq::DONTWAIT)
                .is_ok(),
            Side::In(identity) => match router {
                Some(router) => router
                    .send_multipart(
                        std::iter::once(identity.as_slice()).chain(frames.iter().copied()),
                        zmq::DONTWAIT,
                    )
                    .is_ok(),
                None => false,
            },
        }
    }
}

/// Prefix match with ZMQ SUB semantics (`""` matches everything).
pub(super) fn topic_wanted(demand: &HashSet<String>, topic: &str) -> bool {
    if demand.is_empty() {
        return false;
    }
    if demand.contains("") || demand.contains(topic) {
        return true;
    }
    demand.iter().any(|prefix| topic.starts_with(prefix))
}
