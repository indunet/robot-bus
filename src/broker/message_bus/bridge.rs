//! Federated XSUB/XPUB forwarder for broker↔broker topic links.
//!
//! Local clients keep the 2-frame wire format `[topic][payload]`. Inter-broker
//! pushes use `[topic][hop_path][payload]` where `hop_path` is a comma-separated
//! list of `broker_id`s (loop prevention). Topic demand is tracked from XPUB
//! subscription messages and used to filter pushes and sync peer SUBs.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zmq::{Context as ZmqContext, Socket, SocketType};

use super::metrics::MessageMetrics;
use super::peer::MessagePeer;
use super::BusConfig;

const HOP_SEP: char = ',';

struct PeerLink {
    /// Push federated 3-frame messages into the peer's XSUB.
    pub_sock: Socket,
    /// Express demand on the peer's XPUB; data copies are drained (push is authoritative).
    sub_sock: Socket,
}

/// Run the federated forwarder until `shutdown` is set.
pub fn run_federated(
    context: &ZmqContext,
    xsub: Socket,
    xpub: Socket,
    config: &BusConfig,
    shutdown: &AtomicBool,
    metrics: Option<Arc<MessageMetrics>>,
) -> Result<()> {
    let broker_id = if config.broker_id.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        config.broker_id.clone()
    };

    xpub.set_xpub_verbose(true)
        .context("enable XPUB verbose subscriptions")?;

    let mut peers = Vec::with_capacity(config.peers.len());
    for peer in &config.peers {
        peers.push(connect_peer(context, peer, config.snd_hwm, config.rcv_hwm)?);
    }

    println!(
        "message_bus federation enabled\n  \
         broker_id: {broker_id}\n  \
         peers: {}",
        config.peers.len()
    );

    let mut subscribed: HashSet<String> = HashSet::new();
    let mut active = true;
    while active && !shutdown.load(Ordering::Acquire) {
        let mut items = Vec::with_capacity(2 + peers.len());
        items.push(xsub.as_poll_item(zmq::POLLIN));
        items.push(xpub.as_poll_item(zmq::POLLIN));
        for link in &peers {
            items.push(link.sub_sock.as_poll_item(zmq::POLLIN));
        }

        if zmq::poll(&mut items, 100).is_err() {
            break;
        }

        if items[0].is_readable() {
            while let Ok(frames) = xsub.recv_multipart(zmq::DONTWAIT) {
                handle_xsub_frames(
                    &frames,
                    &xpub,
                    &peers,
                    &broker_id,
                    &subscribed,
                    metrics.as_ref(),
                )?;
            }
        }

        if items[1].is_readable() {
            while let Ok(frames) = xpub.recv_multipart(zmq::DONTWAIT) {
                if let Some((topic, is_sub)) = parse_subscription(&frames) {
                    let changed = if is_sub {
                        subscribed.insert(topic.clone())
                    } else {
                        subscribed.remove(&topic)
                    };
                    // Forward subscription upstream (XSUB filters publishers / peer PUBs).
                    xsub.send_multipart(frames.iter().map(|f| f.as_slice()), 0)
                        .context("forward subscription to XSUB")?;
                    if changed {
                        sync_peer_subs(&peers, &topic, is_sub)?;
                    }
                }
            }
        }

        for (i, link) in peers.iter().enumerate() {
            if items[2 + i].is_readable() {
                drain_peer_sub(&link.sub_sock);
            }
        }

        active = !shutdown.load(Ordering::Acquire);
    }

    Ok(())
}

fn connect_peer(
    context: &ZmqContext,
    peer: &MessagePeer,
    snd_hwm: i32,
    rcv_hwm: i32,
) -> Result<PeerLink> {
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
    sub_sock.set_rcvhwm(rcv_hwm.max(100)).context("peer SUB rcvhwm")?;
    sub_sock
        .connect(&peer.xpub)
        .with_context(|| format!("connect federation SUB -> {}", peer.xpub))?;

    Ok(PeerLink { pub_sock, sub_sock })
}

fn handle_xsub_frames(
    frames: &[Vec<u8>],
    xpub: &Socket,
    peers: &[PeerLink],
    broker_id: &str,
    subscribed: &HashSet<String>,
    metrics: Option<&Arc<MessageMetrics>>,
) -> Result<()> {
    match frames.len() {
        2 => {
            let topic = std::str::from_utf8(&frames[0]).unwrap_or("");
            deliver_local(xpub, &frames[0], &frames[1], topic, metrics)?;
            if topic_wanted(subscribed, topic) {
                push_to_peers(peers, &frames[0], broker_id.as_bytes(), &frames[1])?;
            }
        }
        3 => {
            let topic = std::str::from_utf8(&frames[0]).unwrap_or("");
            let hops = std::str::from_utf8(&frames[1]).unwrap_or("");
            if hop_contains(hops, broker_id) {
                return Ok(());
            }
            deliver_local(xpub, &frames[0], &frames[2], topic, metrics)?;
            if topic_wanted(subscribed, topic) {
                let new_hops = extend_hops(hops, broker_id);
                push_to_peers(peers, &frames[0], new_hops.as_bytes(), &frames[2])?;
            }
        }
        _ => {
            // Ignore unexpected shapes (e.g. malformed clients).
        }
    }
    Ok(())
}

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

fn push_to_peers(peers: &[PeerLink], topic: &[u8], hops: &[u8], payload: &[u8]) -> Result<()> {
    for link in peers {
        // DONTWAIT: slow / disconnected peers must not stall the local bus.
        let _ = link
            .pub_sock
            .send_multipart([topic, hops, payload], zmq::DONTWAIT);
    }
    Ok(())
}

fn sync_peer_subs(peers: &[PeerLink], topic: &str, subscribe: bool) -> Result<()> {
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

fn topic_wanted(subscribed: &HashSet<String>, topic: &str) -> bool {
    if subscribed.contains("") || subscribed.contains(topic) {
        return true;
    }
    // Prefix subscriptions (ZMQ SUB semantics).
    subscribed.iter().any(|prefix| topic.starts_with(prefix))
}

fn hop_contains(hops: &str, broker_id: &str) -> bool {
    hops.split(HOP_SEP).any(|h| h == broker_id)
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
        assert_eq!(extend_hops("a", "b"), "a,b");
        assert_eq!(extend_hops("", "a"), "a");
    }

    #[test]
    fn topic_prefix_match() {
        let mut s = HashSet::new();
        s.insert("robot/".to_string());
        assert!(topic_wanted(&s, "robot/pose"));
        assert!(!topic_wanted(&s, "other"));
        s.insert(String::new());
        assert!(topic_wanted(&s, "other"));
    }
}
