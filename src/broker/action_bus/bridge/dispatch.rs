//! Goal dispatch, cancel, reply delivery, and pending-queue retry.

use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use zmq::Socket;

use super::protocol::{
    ERR_NO_GOAL, ERR_NO_WORKER, ERR_WORKER_DIED, KIND_CANCEL, KIND_GOAL, KIND_RESULT, MAX_PENDING,
};
use super::remote::{PeerLink, PendingGoal, RemoteActions, RemoteRoute};
use super::super::broker::{
    build_client_reply, build_error_body, build_worker_cancel, build_worker_goal, extend_hops,
    hop_contains, GoalEntry, GoalReply, GoalTable, WorkerRegistry,
};

pub(super) fn select_remote<'a>(
    remote: &'a RemoteActions,
    remote_rr: &mut HashMap<String, usize>,
    action: &str,
    hop_path: &str,
) -> Option<&'a RemoteRoute> {
    let list = remote.by_action.get(action)?;
    let candidates: Vec<usize> = list
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            !hop_contains(hop_path, &r.origin_broker_id)
                && !r
                    .hop_path
                    .split(',')
                    .any(|h| !h.is_empty() && hop_contains(hop_path, h))
        })
        .map(|(i, _)| i)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let cursor = remote_rr.entry(action.to_string()).or_insert(0);
    let pick = candidates[*cursor % candidates.len()];
    *cursor = (*cursor + 1) % candidates.len();
    Some(&list[pick])
}

pub(super) fn dispatch_goal(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    pending: &mut VecDeque<PendingGoal>,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    body: &[u8],
    hop_path: &str,
    reply: GoalReply,
) -> Result<()> {
    let act_str = match std::str::from_utf8(action) {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(()),
    };
    let hops_for_select = if hop_path.is_empty() {
        broker_id.to_string()
    } else {
        hop_path.to_string()
    };

    // Local first.
    if registry.worker_count(&act_str) > 0 {
        if let Some(worker_id) = registry.select_worker(&act_str) {
            goals.insert_full(
                goal_id.to_vec(),
                client_id.to_vec(),
                worker_id.clone(),
                action.to_vec(),
                clone_reply(&reply),
                None,
            );
            // Forward to local worker with a synthetic client id encoding goal_id
            // so replies can always be matched — actually workers echo client_id.
            // We pass client_id as goal bookkeeping; deliver uses GoalTable by goal_id.
            let fwd = build_worker_goal(&worker_id, client_id, action, goal_id, body);
            backend.send_multipart(fwd, 0).context("backend send goal")?;
            return Ok(());
        }
    }

    // Remote.
    if let Some(route) = select_remote(remote, remote_rr, &act_str, &hops_for_select) {
        let peer_idx = route.peer_idx;
        let out_hop = extend_hops(&hops_for_select, broker_id);
        goals.insert_full(
            goal_id.to_vec(),
            client_id.to_vec(),
            Vec::new(),
            action.to_vec(),
            clone_reply(&reply),
            Some(peer_idx),
        );
        let _ = peers[peer_idx].dealer.send_multipart(
            [action, goal_id, KIND_GOAL, out_hop.as_bytes(), body],
            zmq::DONTWAIT,
        );
        return Ok(());
    }

    if pending.len() < MAX_PENDING {
        pending.push_back(PendingGoal {
            client_identity: client_id.to_vec(),
            action: action.to_vec(),
            goal_id: goal_id.to_vec(),
            body: body.to_vec(),
            hop_path: hops_for_select,
            reply: clone_reply(&reply),
            queued_at: Instant::now(),
        });
    } else {
        send_error_result(
            frontend,
            backend,
            peers,
            &reply,
            client_id,
            action,
            goal_id,
            ERR_NO_WORKER,
        )?;
    }
    Ok(())
}

pub(super) fn clone_reply(r: &GoalReply) -> GoalReply {
    match r {
        GoalReply::Frontend => GoalReply::Frontend,
        GoalReply::FedBackend { identity } => GoalReply::FedBackend {
            identity: identity.clone(),
        },
    }
}

pub(super) fn handle_cancel(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    goals: &GoalTable,
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    body: &[u8],
    reply: GoalReply,
) -> Result<()> {
    if let Some(entry) = goals.get(goal_id) {
        if let Some(peer_idx) = entry.via_peer {
            let hop = ""; // cancel follows established goal; hop not needed for routing
            let _ = peers[peer_idx].dealer.send_multipart(
                [action, goal_id, KIND_CANCEL, hop.as_bytes(), body],
                zmq::DONTWAIT,
            );
        } else if !entry.worker_identity.is_empty() {
            let fwd = build_worker_cancel(
                &entry.worker_identity,
                client_id,
                action,
                goal_id,
                body,
            );
            backend
                .send_multipart(fwd, 0)
                .context("backend send cancel")?;
        }
    } else {
        send_error_result(
            frontend,
            backend,
            peers,
            &reply,
            client_id,
            action,
            goal_id,
            ERR_NO_GOAL,
        )?;
    }
    Ok(())
}

pub(super) fn deliver_goal_reply(
    frontend: &Socket,
    backend: &Socket,
    peers: &[PeerLink],
    registry: &mut WorkerRegistry,
    goals: &mut GoalTable,
    goal_id: &[u8],
    action: &[u8],
    kind: &[u8],
    body: &[u8],
) -> Result<()> {
    let Some(entry) = goals.get(goal_id) else {
        return Ok(());
    };
    match &entry.reply {
        GoalReply::Frontend => {
            let reply = build_client_reply(
                &entry.client_identity,
                action,
                goal_id,
                kind,
                body,
            );
            frontend
                .send_multipart(reply, 0)
                .context("frontend send feedback/result")?;
        }
        GoalReply::FedBackend { identity } => {
            // Reply toward the peer that sent us the goal (via backend ROUTER).
            let _ = backend.send_multipart(
                [
                    identity.as_slice(),
                    action,
                    goal_id,
                    kind,
                    b"", // hop unused on return
                    body,
                ],
                zmq::DONTWAIT,
            );
        }
    }

    if kind == KIND_RESULT {
        if let Some(entry) = goals.remove(goal_id) {
            if !entry.worker_identity.is_empty() {
                registry.release_worker(&entry.worker_identity);
            }
        }
    }
    let _ = peers; // peer dealer replies already landed here via handle_peer_dealer
    Ok(())
}

pub(super) fn send_error_result(
    frontend: &Socket,
    backend: &Socket,
    peers: &[PeerLink],
    reply: &GoalReply,
    client_id: &[u8],
    action: &[u8],
    goal_id: &[u8],
    err_prefix: &[u8],
) -> Result<()> {
    let err = build_error_body(err_prefix, action);
    match reply {
        GoalReply::Frontend => {
            let msg = build_client_reply(client_id, action, goal_id, KIND_RESULT, &err);
            frontend
                .send_multipart(msg, 0)
                .context("frontend send error result")?;
        }
        GoalReply::FedBackend { identity } => {
            let _ = backend.send_multipart(
                [
                    identity.as_slice(),
                    action,
                    goal_id,
                    KIND_RESULT,
                    b"",
                    err.as_slice(),
                ],
                zmq::DONTWAIT,
            );
        }
    }
    let _ = peers;
    Ok(())
}

pub(super) fn send_died_results(
    frontend: &Socket,
    backend: &Socket,
    dropped: Vec<GoalEntry>,
) -> Result<()> {
    for e in dropped {
        let err = build_error_body(ERR_WORKER_DIED, &e.action);
        match &e.reply {
            GoalReply::Frontend => {
                let reply = build_client_reply(
                    &e.client_identity,
                    &e.action,
                    &e.goal_id,
                    KIND_RESULT,
                    &err,
                );
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send worker-died")?;
            }
            GoalReply::FedBackend { identity } => {
                let _ = backend.send_multipart(
                    [
                        identity.as_slice(),
                        e.action.as_slice(),
                        e.goal_id.as_slice(),
                        KIND_RESULT,
                        b"",
                        err.as_slice(),
                    ],
                    zmq::DONTWAIT,
                );
            }
        }
    }
    Ok(())
}

pub(super) fn retry_pending(
    frontend: &Socket,
    backend: &Socket,
    peers: &mut [PeerLink],
    pending: &mut VecDeque<PendingGoal>,
    registry: &mut WorkerRegistry,
    remote: &mut RemoteActions,
    goals: &mut GoalTable,
    remote_rr: &mut HashMap<String, usize>,
    broker_id: &str,
    now: Instant,
    pending_timeout: Duration,
) -> Result<()> {
    let mut still = VecDeque::new();
    while let Some(req) = pending.pop_front() {
        let act_str = match std::str::from_utf8(&req.action) {
            Ok(s) => s.to_string(),
            Err(_) => continue,
        };
        let mut dispatched = false;
        if registry.worker_count(&act_str) > 0 {
            if let Some(worker_id) = registry.select_worker(&act_str) {
                goals.insert_full(
                    req.goal_id.clone(),
                    req.client_identity.clone(),
                    worker_id.clone(),
                    req.action.clone(),
                    clone_reply(&req.reply),
                    None,
                );
                let fwd = build_worker_goal(
                    &worker_id,
                    &req.client_identity,
                    &req.action,
                    &req.goal_id,
                    &req.body,
                );
                backend
                    .send_multipart(fwd, 0)
                    .context("backend send pending goal")?;
                dispatched = true;
            }
        }
        if !dispatched {
            if let Some(route) = select_remote(remote, remote_rr, &act_str, &req.hop_path) {
                let peer_idx = route.peer_idx;
                let out_hop = extend_hops(&req.hop_path, broker_id);
                goals.insert_full(
                    req.goal_id.clone(),
                    req.client_identity.clone(),
                    Vec::new(),
                    req.action.clone(),
                    clone_reply(&req.reply),
                    Some(peer_idx),
                );
                let _ = peers[peer_idx].dealer.send_multipart(
                    [
                        req.action.as_slice(),
                        req.goal_id.as_slice(),
                        KIND_GOAL,
                        out_hop.as_bytes(),
                        req.body.as_slice(),
                    ],
                    zmq::DONTWAIT,
                );
                dispatched = true;
            }
        }
        if !dispatched {
            if now.duration_since(req.queued_at) > pending_timeout {
                send_error_result(
                    frontend,
                    backend,
                    peers,
                    &req.reply,
                    &req.client_identity,
                    &req.action,
                    &req.goal_id,
                    ERR_NO_WORKER,
                )?;
            } else {
                still.push_back(req);
            }
        }
    }
    *pending = still;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Instant;

    #[test]
    fn remote_actions_upsert_and_select() {
        let mut r = RemoteActions::new();
        let now = Instant::now();
        r.upsert("act.x".into(), 0, "a".into(), "a".into(), now);
        r.upsert("act.x".into(), 1, "c".into(), "b,c".into(), now);
        let mut rr = HashMap::new();
        let sel = select_remote(&r, &mut rr, "act.x", "a").unwrap();
        assert_eq!(sel.peer_idx, 1);
        assert!(select_remote(&r, &mut rr, "act.x", "a,c").is_none());
    }

    #[test]
    fn hop_helpers() {
        assert!(hop_contains("a,b", "b"));
        assert_eq!(extend_hops("a", "b"), "a,b");
    }
}
