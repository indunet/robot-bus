//! Peer advertisement sync and federation heartbeats.

use anyhow::Result;
use std::collections::HashMap;

use super::protocol::{CMD_DISCONNECT, CMD_HEARTBEAT, CMD_READY_FED};
use super::remote::PeerLink;
use super::super::broker::{hop_contains, WorkerRegistry};

pub(super) fn sync_all_advertisements(
    peers: &mut [PeerLink],
    registry: &WorkerRegistry,
    broker_id: &str,
) -> Result<()> {
    let snapshot = registry.advertise_snapshot(broker_id);
    for link in peers.iter_mut() {
        sync_peer_advertisement(link, &snapshot, broker_id)?;
    }
    Ok(())
}

fn sync_peer_advertisement(
    link: &mut PeerLink,
    snapshot: &[(String, String, String, String)],
    broker_id: &str,
) -> Result<()> {
    let peer_id = link.peer_broker_id.clone();
    let mut desired: HashMap<String, (String, String)> = HashMap::new();

    for (svc, origin, hop, via) in snapshot {
        if !via.is_empty() {
            match &peer_id {
                Some(pid) if via == pid || hop_contains(hop, pid) => continue,
                None => continue, // wait until we know peer id before re-advertising
                _ => {}
            }
        } else if let Some(pid) = &peer_id {
            // Local offering: still skip if somehow hop already contains peer.
            if hop_contains(hop, pid) {
                continue;
            }
        }

        desired
            .entry(svc.clone())
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
    for svc in stale {
        let _ = link
            .dealer
            .send_multipart([CMD_DISCONNECT, svc.as_bytes()], zmq::DONTWAIT);
        link.advertised.remove(&svc);
    }

    for (svc, (origin, hop)) in &desired {
        let need_ready = match link.advertised.get(svc) {
            Some((o, h)) if o == origin && h == hop => false,
            _ => true,
        };
        if need_ready {
            let _ = link.dealer.send_multipart(
                [
                    CMD_READY_FED,
                    svc.as_bytes(),
                    origin.as_bytes(),
                    hop.as_bytes(),
                ],
                zmq::DONTWAIT,
            );
            link.advertised
                .insert(svc.clone(), (origin.clone(), hop.clone()));
        }
    }
    Ok(())
}

pub(super) fn send_peer_heartbeats(peers: &[PeerLink]) -> Result<()> {
    for link in peers {
        for svc in link.advertised.keys() {
            let _ = link
                .dealer
                .send_multipart([CMD_HEARTBEAT, svc.as_bytes()], zmq::DONTWAIT);
        }
    }
    Ok(())
}
