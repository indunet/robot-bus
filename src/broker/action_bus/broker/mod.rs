//! Action bus broker: dual-ROUTER broker with worker registry and goal table.
//!
//! Mirrors the ROS 2 action semantics: a client sends a goal, then receives
//! zero or more feedback messages, and finally one result. Unlike the service
//! bus (REQ client, one response), the action client is a DEALER so it can
//! receive multiple responses for the same goal.
//!
//! The broker parses only the UTF-8 `action_name`, `goal_id` (UTF-8), and the
//! `GOAL/FEEDBACK/RESULT/CANCEL` control tokens. The `body` frame is forwarded
//! as opaque bytes — no protobuf dependency.

mod goals;
mod registry;
mod wire;

pub use goals::GoalTable;
pub(crate) use goals::{GoalEntry, GoalReply, extend_hops, hop_contains};
pub use registry::WorkerRegistry;
pub use wire::{
    ClientKind, WorkerControl, WorkerKind, WorkerMessage, build_client_reply, build_error_body,
    build_worker_cancel, build_worker_goal, parse_client_message, parse_worker_message,
};

use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use zmq::Socket;

use super::ActionBusConfig;
use super::metrics::ActionMetrics;
use goals::PendingGoal;
use wire::{
    ERR_CANCELLED, ERR_NO_GOAL, ERR_NO_WORKER, ERR_WORKER_DIED, KIND_RESULT, MAX_GOALS,
    MAX_PENDING, POLL_CAP_MS, kind_as_bytes,
};

/// Run the broker poll loop until `shutdown` is set.
pub fn run_loop(
    frontend: &Socket,
    backend: &Socket,
    config: &ActionBusConfig,
    shutdown: &AtomicBool,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let mut registry = WorkerRegistry::new();
    let mut goals = GoalTable::new();
    let mut pending: VecDeque<PendingGoal> = VecDeque::new();
    let mut next_sweep = Instant::now() + Duration::from_millis(config.heartbeat_interval_ms);

    loop {
        if shutdown.load(Ordering::Acquire) {
            break;
        }

        let sweep_in = next_sweep.saturating_duration_since(Instant::now());
        let timeout_ms = (sweep_in.as_millis() as i64).min(POLL_CAP_MS).max(0);
        let mut items = [
            frontend.as_poll_item(zmq::POLLIN),
            backend.as_poll_item(zmq::POLLIN),
        ];
        zmq::poll(&mut items, timeout_ms).context("poll")?;

        if items[0].get_revents().contains(zmq::POLLIN) {
            handle_client_message(
                frontend,
                backend,
                &mut registry,
                &mut goals,
                &mut pending,
                metrics,
            )?;
        }
        if items[1].get_revents().contains(zmq::POLLIN) {
            handle_worker_message(backend, frontend, &mut registry, &mut goals, metrics)?;
        }

        if Instant::now() >= next_sweep {
            let now = Instant::now();
            let dead = registry.sweep_dead(now, Duration::from_millis(config.heartbeat_timeout_ms));
            for (wid, act) in dead {
                if let Some(m) = metrics {
                    m.set_workers(&act, registry.worker_count(&act) as u64);
                }
                reclaim_worker_goals(frontend, &mut goals, &wid, metrics)?;
            }
            retry_pending(
                frontend,
                backend,
                &mut pending,
                &mut registry,
                &mut goals,
                now,
                Duration::from_millis(config.pending_timeout_ms),
                metrics,
            )?;
            next_sweep = now + Duration::from_millis(config.heartbeat_interval_ms);
        }
    }

    Ok(())
}

/// Send a synthetic WORKER_DIED result to each client whose goal was owned by
/// `worker_identity`, and drop those goals from the table.
fn reclaim_worker_goals(
    frontend: &Socket,
    goals: &mut GoalTable,
    worker_identity: &[u8],
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let dropped = goals.evict_worker(worker_identity);
    for e in dropped {
        if let Some(m) = metrics {
            let name = String::from_utf8_lossy(&e.action);
            m.record_error(&name, Some(&e.goal_id));
            m.record_worker_died(&name);
        }
        let err = build_error_body(ERR_WORKER_DIED, &e.action);
        let reply =
            build_client_reply(&e.client_identity, &e.action, &e.goal_id, KIND_RESULT, &err);
        frontend
            .send_multipart(reply, 0)
            .context("frontend send worker-died result")?;
    }
    Ok(())
}

fn handle_client_message(
    frontend: &Socket,
    backend: &Socket,
    registry: &mut WorkerRegistry,
    goals: &mut GoalTable,
    pending: &mut VecDeque<PendingGoal>,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let frames = match frontend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("frontend recv_multipart"),
    };
    // ROUTER prepends client_id; DEALER does not insert an empty delimiter.
    // So frames: [client_id][action][goal_id][kind][body] (5 frames).
    if frames.len() < 5 {
        return Ok(()); // malformed, drop
    }
    let client_id = &frames[0];
    let rest = &frames[1..5];
    let msg = match parse_client_message(rest) {
        Some(m) => m,
        None => return Ok(()), // unknown kind or malformed, drop
    };
    match msg.kind {
        ClientKind::Goal => {
            let action_str = match std::str::from_utf8(msg.action) {
                Ok(s) => s.to_string(),
                Err(_) => return Ok(()),
            };
            if goals.contains(msg.goal_id) || goals.len() >= MAX_GOALS {
                if let Some(m) = metrics {
                    m.record_error(&action_str, Some(msg.goal_id));
                }
                // Duplicate / capacity: reject with NO_GOAL so the client unblocks.
                let err = build_error_body(ERR_NO_GOAL, msg.action);
                let reply =
                    build_client_reply(client_id, msg.action, msg.goal_id, KIND_RESULT, &err);
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send duplicate/cap reject")?;
                return Ok(());
            }
            if let Some(worker_id) = registry.select_worker(&action_str) {
                if !goals.try_insert_full(
                    msg.goal_id.to_vec(),
                    client_id.to_vec(),
                    worker_id.clone(),
                    msg.action.to_vec(),
                    GoalReply::Frontend,
                    None,
                    MAX_GOALS,
                ) {
                    let err = build_error_body(ERR_NO_GOAL, msg.action);
                    let reply =
                        build_client_reply(client_id, msg.action, msg.goal_id, KIND_RESULT, &err);
                    frontend
                        .send_multipart(reply, 0)
                        .context("frontend send insert reject")?;
                    return Ok(());
                }
                if let Some(m) = metrics {
                    m.record_run_start(&action_str, msg.goal_id);
                }
                let fwd =
                    build_worker_goal(&worker_id, client_id, msg.action, msg.goal_id, msg.body);
                backend
                    .send_multipart(fwd, 0)
                    .context("backend send goal")?;
            } else if pending.len() < MAX_PENDING {
                pending.push_back(PendingGoal {
                    client_identity: client_id.to_vec(),
                    action: msg.action.to_vec(),
                    goal_id: msg.goal_id.to_vec(),
                    body: msg.body.to_vec(),
                    queued_at: Instant::now(),
                });
            } else {
                if let Some(m) = metrics {
                    m.record_error(&action_str, None);
                }
                let err = build_error_body(ERR_NO_WORKER, msg.action);
                let reply =
                    build_client_reply(client_id, msg.action, msg.goal_id, KIND_RESULT, &err);
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send reject")?;
            }
        }
        ClientKind::Cancel => {
            // Cancel a still-queued goal before it is dispatched.
            if let Some(pos) = pending.iter().position(|p| p.goal_id == msg.goal_id) {
                let req = pending.remove(pos).expect("position just found");
                if let Some(m) = metrics {
                    if let Ok(name) = std::str::from_utf8(&req.action) {
                        m.record_error(name, Some(&req.goal_id));
                        m.record_cancelled(name);
                    }
                }
                let err = build_error_body(ERR_CANCELLED, &req.action);
                let reply = build_client_reply(
                    &req.client_identity,
                    &req.action,
                    &req.goal_id,
                    KIND_RESULT,
                    &err,
                );
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send cancelled")?;
                return Ok(());
            }
            // Route the cancel to the worker currently owning this goal.
            if let Some(entry) = goals.get(msg.goal_id) {
                let fwd = build_worker_cancel(
                    &entry.worker_identity,
                    client_id,
                    msg.action,
                    msg.goal_id,
                    msg.body,
                );
                backend
                    .send_multipart(fwd, 0)
                    .context("backend send cancel")?;
            } else {
                // Goal unknown or already finished: synthesize a RESULT so the
                // client's DEALER is unlocked. Body encodes the reason.
                if let Some(m) = metrics {
                    if let Ok(name) = std::str::from_utf8(msg.action) {
                        m.record_error(name, None);
                    }
                }
                let err = build_error_body(ERR_NO_GOAL, msg.action);
                let reply =
                    build_client_reply(client_id, msg.action, msg.goal_id, KIND_RESULT, &err);
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send cancel-no-goal")?;
            }
        }
    }
    Ok(())
}

fn handle_worker_message(
    backend: &Socket,
    frontend: &Socket,
    registry: &mut WorkerRegistry,
    goals: &mut GoalTable,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let frames = match backend.recv_multipart(0) {
        Ok(f) => f,
        Err(zmq::Error::EAGAIN) => return Ok(()),
        Err(e) => return Err(e).context("backend recv_multipart"),
    };
    match parse_worker_message(&frames) {
        Some(WorkerMessage::Control {
            worker_id,
            action,
            control,
        }) => {
            match control {
                WorkerControl::Ready => {
                    let act_str = String::from_utf8_lossy(action).into_owned();
                    registry.register(worker_id.to_vec(), act_str.clone(), Instant::now());
                    if let Some(m) = metrics {
                        m.set_workers(&act_str, registry.worker_count(&act_str) as u64);
                    }
                }
                WorkerControl::Heartbeat => {
                    registry.heartbeat(worker_id, Instant::now());
                }
                WorkerControl::Disconnect => {
                    let act = registry.remove(worker_id);
                    reclaim_worker_goals(frontend, goals, worker_id, metrics)?;
                    if let (Some(m), Some(act)) = (metrics, act) {
                        m.set_workers(&act, registry.worker_count(&act) as u64);
                    }
                }
            }
            Ok(())
        }
        Some(WorkerMessage::Response {
            worker_id,
            client_id,
            action,
            goal_id,
            kind,
            body,
        }) => {
            // Fence late replies after WORKER_DIED reclaim (or unknown goal_id).
            let Some(entry) = goals.get(goal_id) else {
                return Ok(());
            };
            if entry.worker_identity != worker_id {
                return Ok(());
            }
            let reply = build_client_reply(client_id, action, goal_id, kind_as_bytes(kind), body);
            frontend
                .send_multipart(reply, 0)
                .context("frontend send feedback/result")?;
            if kind == WorkerKind::Result {
                registry.release_worker(worker_id);
                goals.remove(goal_id);
                if let Some(m) = metrics {
                    if let Ok(name) = std::str::from_utf8(action) {
                        m.record_run_ok(name, goal_id);
                    }
                }
            }
            Ok(())
        }
        None => Ok(()), // unknown shape, drop
    }
}

/// Retry queued goals; give up (send NO_WORKER) for those stale beyond timeout.
fn retry_pending(
    frontend: &Socket,
    backend: &Socket,
    pending: &mut VecDeque<PendingGoal>,
    registry: &mut WorkerRegistry,
    goals: &mut GoalTable,
    now: Instant,
    pending_timeout: Duration,
    metrics: Option<&Arc<ActionMetrics>>,
) -> Result<()> {
    let mut still_pending = VecDeque::new();
    while let Some(req) = pending.pop_front() {
        let act_str = match std::str::from_utf8(&req.action) {
            Ok(s) => s.to_string(),
            Err(_) => continue, // drop malformed
        };
        if let Some(worker_id) = registry.select_worker(&act_str) {
            if !goals.try_insert_full(
                req.goal_id.clone(),
                req.client_identity.clone(),
                worker_id.clone(),
                req.action.clone(),
                GoalReply::Frontend,
                None,
                MAX_GOALS,
            ) {
                // Capacity / duplicate — reject to the waiting client.
                if let Some(m) = metrics {
                    m.record_error(&act_str, Some(&req.goal_id));
                }
                let err = build_error_body(ERR_NO_GOAL, &req.action);
                let reply = build_client_reply(
                    &req.client_identity,
                    &req.action,
                    &req.goal_id,
                    KIND_RESULT,
                    &err,
                );
                frontend
                    .send_multipart(reply, 0)
                    .context("frontend send pending insert reject")?;
                continue;
            }
            if let Some(m) = metrics {
                m.record_run_start(&act_str, &req.goal_id);
            }
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
        } else if now.duration_since(req.queued_at) > pending_timeout {
            if let Some(m) = metrics {
                m.record_error(&act_str, None);
            }
            let err = build_error_body(ERR_NO_WORKER, &req.action);
            let reply = build_client_reply(
                &req.client_identity,
                &req.action,
                &req.goal_id,
                KIND_RESULT,
                &err,
            );
            frontend
                .send_multipart(reply, 0)
                .context("frontend send pending reject")?;
        } else {
            still_pending.push_back(req);
        }
    }
    *pending = still_pending;
    Ok(())
}

// ── Unit tests (no sockets, no ports) ─────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Instant {
        Instant::now()
    }

    // ── WorkerRegistry ──

    #[test]
    fn registry_register_and_select() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "act.x".into(), t);
        assert_eq!(r.worker_count("act.x"), 1);
        assert!(r.is_alive(b"w1"));
        let picked = r.select_worker("act.x");
        assert_eq!(picked, Some(b"w1".to_vec()));
    }

    #[test]
    fn registry_select_none_when_empty() {
        let mut r = WorkerRegistry::new();
        assert_eq!(r.select_worker("act.missing"), None);
        assert_eq!(r.worker_count("act.missing"), 0);
    }

    #[test]
    fn registry_round_robin_balances() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "act".into(), t);
        r.register(b"w2".to_vec(), "act".into(), t);
        let first = r.select_worker("act").unwrap();
        let second = r.select_worker("act").unwrap();
        let third = r.select_worker("act").unwrap();
        assert_eq!(first, b"w1".to_vec());
        assert_eq!(second, b"w2".to_vec());
        assert_eq!(third, b"w1".to_vec());
    }

    #[test]
    fn registry_release_decrements() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "act".into(), t);
        let _ = r.select_worker("act");
        r.release_worker(b"w1");
        assert_eq!(r.select_worker("act"), Some(b"w1".to_vec()));
    }

    #[test]
    fn registry_sweep_evicts_dead() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "act".into(), t);
        let dead = r.sweep_dead(t + Duration::from_secs(10), Duration::from_secs(1));
        assert_eq!(dead, vec![(b"w1".to_vec(), "act".into())]);
        assert!(!r.is_alive(b"w1"));
        assert_eq!(r.worker_count("act"), 0);
    }

    #[test]
    fn registry_reregister_moves_action() {
        let mut r = WorkerRegistry::new();
        let t = now();
        r.register(b"w1".to_vec(), "act.a".into(), t);
        r.register(b"w1".to_vec(), "act.b".into(), t);
        assert_eq!(r.worker_count("act.a"), 0);
        assert_eq!(r.worker_count("act.b"), 1);
        assert_eq!(r.select_worker("act.b"), Some(b"w1".to_vec()));
    }

    // ── GoalTable ──

    #[test]
    fn goal_table_insert_remove() {
        let mut g = GoalTable::new();
        g.insert(
            b"g1".to_vec(),
            b"c1".to_vec(),
            b"w1".to_vec(),
            b"act".to_vec(),
        );
        assert_eq!(g.len(), 1);
        assert!(g.get(b"g1").is_some());
        let e = g.remove(b"g1").unwrap();
        assert_eq!(e.client_identity, b"c1");
        assert_eq!(e.worker_identity, b"w1");
        assert_eq!(e.goal_id, b"g1");
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn goal_table_evict_worker_drops_owned() {
        let mut g = GoalTable::new();
        g.insert(
            b"g1".to_vec(),
            b"c1".to_vec(),
            b"w1".to_vec(),
            b"act".to_vec(),
        );
        g.insert(
            b"g2".to_vec(),
            b"c2".to_vec(),
            b"w2".to_vec(),
            b"act".to_vec(),
        );
        g.insert(
            b"g3".to_vec(),
            b"c3".to_vec(),
            b"w1".to_vec(),
            b"act".to_vec(),
        );
        let dropped = g.evict_worker(b"w1");
        assert_eq!(dropped.len(), 2);
        assert!(g.get(b"g1").is_none());
        assert!(g.get(b"g3").is_none());
        assert!(g.get(b"g2").is_some());
    }

    #[test]
    fn goal_table_evict_unknown_worker_is_empty() {
        let mut g = GoalTable::new();
        g.insert(
            b"g1".to_vec(),
            b"c1".to_vec(),
            b"w1".to_vec(),
            b"act".to_vec(),
        );
        assert!(g.evict_worker(b"wX").is_empty());
        assert_eq!(g.len(), 1);
    }

    // ── Frame parsing ──

    #[test]
    fn parse_client_goal() {
        let frames = vec![
            b"act.x".to_vec(),
            b"g1".to_vec(),
            b"GOAL".to_vec(),
            b"body".to_vec(),
        ];
        let m = parse_client_message(&frames).unwrap();
        assert_eq!(m.action, b"act.x");
        assert_eq!(m.goal_id, b"g1");
        assert_eq!(m.kind, ClientKind::Goal);
    }

    #[test]
    fn parse_client_cancel() {
        let frames = vec![
            b"act.x".to_vec(),
            b"g1".to_vec(),
            b"CANCEL".to_vec(),
            b"".to_vec(),
        ];
        let m = parse_client_message(&frames).unwrap();
        assert_eq!(m.kind, ClientKind::Cancel);
    }

    #[test]
    fn parse_client_rejects_feedback_kind() {
        let frames = vec![
            b"act.x".to_vec(),
            b"g1".to_vec(),
            b"FEEDBACK".to_vec(),
            b"x".to_vec(),
        ];
        assert!(parse_client_message(&frames).is_none());
    }

    #[test]
    fn parse_client_rejects_short() {
        assert!(parse_client_message(&[]).is_none());
        assert!(parse_client_message(&[b"x".to_vec()]).is_none());
    }

    #[test]
    fn parse_client_rejects_empty_action_or_goal() {
        let frames = vec![
            b"".to_vec(),
            b"g1".to_vec(),
            b"GOAL".to_vec(),
            b"b".to_vec(),
        ];
        assert!(parse_client_message(&frames).is_none());
        let frames = vec![
            b"act".to_vec(),
            b"".to_vec(),
            b"GOAL".to_vec(),
            b"b".to_vec(),
        ];
        assert!(parse_client_message(&frames).is_none());
    }

    #[test]
    fn parse_worker_control_ready() {
        let frames = vec![b"w1".to_vec(), b"READY".to_vec(), b"act.x".to_vec()];
        match parse_worker_message(&frames).unwrap() {
            WorkerMessage::Control {
                control, action, ..
            } => {
                assert_eq!(control, WorkerControl::Ready);
                assert_eq!(action, b"act.x");
            }
            _ => panic!("expected control"),
        }
    }

    #[test]
    fn parse_worker_control_heartbeat() {
        let frames = vec![b"w1".to_vec(), b"HEARTBEAT".to_vec(), b"act.x".to_vec()];
        match parse_worker_message(&frames).unwrap() {
            WorkerMessage::Control { control, .. } => {
                assert_eq!(control, WorkerControl::Heartbeat);
            }
            _ => panic!("expected control"),
        }
    }

    #[test]
    fn parse_worker_control_disconnect() {
        let frames = vec![b"w1".to_vec(), b"DISCONNECT".to_vec(), b"act.x".to_vec()];
        match parse_worker_message(&frames).unwrap() {
            WorkerMessage::Control { control, .. } => {
                assert_eq!(control, WorkerControl::Disconnect);
            }
            _ => panic!("expected control"),
        }
    }

    #[test]
    fn parse_worker_feedback() {
        let frames = vec![
            b"w1".to_vec(),
            b"c1".to_vec(),
            b"act.x".to_vec(),
            b"g1".to_vec(),
            b"FEEDBACK".to_vec(),
            b"fb".to_vec(),
        ];
        match parse_worker_message(&frames).unwrap() {
            WorkerMessage::Response { kind, body, .. } => {
                assert_eq!(kind, WorkerKind::Feedback);
                assert_eq!(body, b"fb");
            }
            _ => panic!("expected response"),
        }
    }

    #[test]
    fn parse_worker_result() {
        let frames = vec![
            b"w1".to_vec(),
            b"c1".to_vec(),
            b"act.x".to_vec(),
            b"g1".to_vec(),
            b"RESULT".to_vec(),
            b"done".to_vec(),
        ];
        match parse_worker_message(&frames).unwrap() {
            WorkerMessage::Response { kind, body, .. } => {
                assert_eq!(kind, WorkerKind::Result);
                assert_eq!(body, b"done");
            }
            _ => panic!("expected response"),
        }
    }

    #[test]
    fn parse_worker_rejects_unknown_kind() {
        let frames = vec![
            b"w1".to_vec(),
            b"c1".to_vec(),
            b"act.x".to_vec(),
            b"g1".to_vec(),
            b"WAT".to_vec(),
            b"x".to_vec(),
        ];
        assert!(parse_worker_message(&frames).is_none());
    }

    #[test]
    fn parse_worker_rejects_wrong_frame_count() {
        assert!(parse_worker_message(&[]).is_none());
        assert!(parse_worker_message(&[b"x".to_vec(), b"y".to_vec()]).is_none());
        assert!(
            parse_worker_message(&[
                b"w".to_vec(),
                b"c".to_vec(),
                b"a".to_vec(),
                b"g".to_vec(),
                b"RESULT".to_vec(),
                b"b".to_vec(),
                b"extra".to_vec(),
            ])
            .is_none()
        );
    }

    // ── Frame building ──

    #[test]
    fn build_worker_goal_order() {
        let f = build_worker_goal(b"wid", b"cid", b"act", b"gid", b"body");
        assert_eq!(
            f,
            vec![
                b"wid".to_vec(),
                b"cid".to_vec(),
                b"act".to_vec(),
                b"gid".to_vec(),
                b"GOAL".to_vec(),
                b"body".to_vec(),
            ]
        );
    }

    #[test]
    fn build_worker_cancel_order() {
        let f = build_worker_cancel(b"wid", b"cid", b"act", b"gid", b"body");
        assert_eq!(f[4], b"CANCEL".to_vec());
    }

    #[test]
    fn build_client_reply_order() {
        let r = build_client_reply(b"cid", b"act", b"gid", b"FEEDBACK", b"fb");
        assert_eq!(
            r,
            vec![
                b"cid".to_vec(),
                b"act".to_vec(),
                b"gid".to_vec(),
                b"FEEDBACK".to_vec(),
                b"fb".to_vec(),
            ]
        );
    }

    #[test]
    fn build_error_body_format() {
        let err = build_error_body(ERR_NO_WORKER, b"act.x");
        assert_eq!(&err[..9], b"NO_WORKER");
        assert_eq!(err[9], 0);
        assert_eq!(&err[10..], b"act.x");
    }

    #[test]
    fn build_error_body_worker_died() {
        let err = build_error_body(ERR_WORKER_DIED, b"act.x");
        assert_eq!(&err[..11], b"WORKER_DIED");
        assert_eq!(err[11], 0);
    }
}
