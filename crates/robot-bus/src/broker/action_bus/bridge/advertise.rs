//! Peer advertisement sync and federation heartbeats.

use anyhow::Result;
use std::collections::HashMap;
use crate::zmq::Socket;

use super::super::broker::{WorkerRegistry, hop_contains};
use super::protocol::{CMD_DISCONNECT, CMD_HEARTBEAT, CMD_READY_FED};
use super::remote::{LinkKind, PeerLink, RemoteActions};

pub(super) fn sync_all_advertisements(
    peers: &mut [PeerLink],
    backend: &Socket,
    registry: &WorkerRegistry,
    remote: &RemoteActions,
    broker_id: &str,
) -> Result<()> {
    let mut snapshot: Vec<(String, String, String, Option<usize>)> = Vec::new();
    for act in registry.action_names() {
        snapshot.push((act, broker_id.to_string(), broker_id.to_string(), None));
    }
    for (act, origin, hop, via) in remote.advertise_snapshot(broker_id) {
        snapshot.push((act, origin, hop, Some(via)));
    }

    for (idx, link) in peers.iter_mut().enumerate() {
        if link.enabled() {
            sync_peer_advertisement(link, backend, idx, &snapshot, broker_id)?;
        }
    }
    Ok(())
}

fn sync_peer_advertisement(
    link: &mut PeerLink,
    backend: &Socket,
    peer_idx: usize,
    snapshot: &[(String, String, String, Option<usize>)],
    broker_id: &str,
) -> Result<()> {
    let peer_id = link.peer_broker_id.clone();
    let mut desired: HashMap<String, (String, String)> = HashMap::new();

    for (act, origin, hop, via) in snapshot {
        if let Some(via_idx) = via {
            if *via_idx == peer_idx {
                continue; // don't advertise back to source peer
            }
        }
        if let Some(ref pid) = peer_id {
            if hop_contains(hop, pid) {
                continue;
            }
            if via.is_some() && origin != broker_id {
                // already filtered by via_idx; also skip if hop contains peer
            }
        } else if via.is_some() {
            continue; // wait until peer id known before re-advertising remotes
        }

        desired
            .entry(act.clone())
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
    for act in stale {
        // Retry DISCONNECT on EAGAIN (peer DEALER not connected yet).
        if link.send(backend, &[CMD_DISCONNECT, act.as_bytes()]) {
            link.advertised.remove(&act);
        }
    }

    for (act, (origin, hop)) in &desired {
        let need = match link.advertised.get(act) {
            Some((o, h)) if o == origin && h == hop => false,
            _ => true,
        };
        if need {
            // Cache only after accept; ZMQ_IMMEDIATE drops otherwise and
            // caching would permanently suppress READY_FED retries.
            if link.send(
                backend,
                &[
                    CMD_READY_FED,
                    act.as_bytes(),
                    origin.as_bytes(),
                    hop.as_bytes(),
                ],
            ) {
                link.advertised
                    .insert(act.clone(), (origin.clone(), hop.clone()));
            }
        }
    }
    Ok(())
}

/// Keep the peer's view of our advertisements alive.
///
/// Legacy links send one `HEARTBEAT` per action. Joining links re-send the whole
/// `READY_FED` set instead: it doubles as heartbeat and heals state the peer lost.
pub(super) fn send_peer_heartbeats(peers: &[PeerLink], backend: &Socket) -> Result<()> {
    for link in peers {
        if !link.enabled() {
            continue;
        }
        let refresh = link.join || matches!(link.kind, LinkKind::In(_));
        for (act, (origin, hop)) in &link.advertised {
            if refresh {
                link.send(
                    backend,
                    &[
                        CMD_READY_FED,
                        act.as_bytes(),
                        origin.as_bytes(),
                        hop.as_bytes(),
                    ],
                );
            } else {
                link.send(backend, &[CMD_HEARTBEAT, act.as_bytes()]);
            }
        }
    }
    Ok(())
}
