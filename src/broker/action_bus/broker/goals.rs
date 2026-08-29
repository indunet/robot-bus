//! In-flight goal table and hop-path helpers.

use std::collections::HashMap;
use std::time::Instant;

use super::wire::MAX_GOALS;

/// A goal waiting for an available worker.
pub(super) struct PendingGoal {
    pub(super) client_identity: Vec<u8>,
    pub(super) action: Vec<u8>,
    pub(super) goal_id: Vec<u8>,
    pub(super) body: Vec<u8>,
    pub(super) queued_at: Instant,
}

/// Tracks an in-flight goal so that CANCEL and worker-death can be routed.
pub(crate) struct GoalEntry {
    pub(crate) client_identity: Vec<u8>,
    pub(crate) worker_identity: Vec<u8>,
    pub(crate) action: Vec<u8>,
    pub(crate) goal_id: Vec<u8>,
    /// Where FEEDBACK/RESULT should be delivered (federation).
    pub(crate) reply: GoalReply,
    /// If the goal was forwarded to a remote peer, its PeerLink index.
    pub(crate) via_peer: Option<usize>,
}

/// Where goal replies are delivered.
#[derive(Clone, Debug)]
pub(crate) enum GoalReply {
    /// Local frontend client.
    Frontend,
    /// Federated peer identity on our backend ROUTER (inbound goal from that peer).
    FedBackend { identity: Vec<u8> },
}

/// goal_id -> GoalEntry. Drives CANCEL routing and worker-death recovery.
pub struct GoalTable {
    goals: HashMap<Vec<u8>, GoalEntry>,
}

impl GoalTable {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
        }
    }

    pub fn contains(&self, goal_id: &[u8]) -> bool {
        self.goals.contains_key(goal_id)
    }

    pub fn len(&self) -> usize {
        self.goals.len()
    }

    #[allow(dead_code)]
    pub fn insert(
        &mut self,
        goal_id: Vec<u8>,
        client_identity: Vec<u8>,
        worker_identity: Vec<u8>,
        action: Vec<u8>,
    ) {
        let _ = self.insert_full(
            goal_id,
            client_identity,
            worker_identity,
            action,
            GoalReply::Frontend,
            None,
        );
    }

    /// Insert a new goal. Returns `false` if `goal_id` already exists or the table is full.
    pub(crate) fn try_insert_full(
        &mut self,
        goal_id: Vec<u8>,
        client_identity: Vec<u8>,
        worker_identity: Vec<u8>,
        action: Vec<u8>,
        reply: GoalReply,
        via_peer: Option<usize>,
        max_goals: usize,
    ) -> bool {
        if self.goals.contains_key(&goal_id) || self.goals.len() >= max_goals {
            return false;
        }
        self.goals.insert(
            goal_id.clone(),
            GoalEntry {
                client_identity,
                worker_identity,
                action,
                goal_id,
                reply,
                via_peer,
            },
        );
        true
    }

    pub(crate) fn insert_full(
        &mut self,
        goal_id: Vec<u8>,
        client_identity: Vec<u8>,
        worker_identity: Vec<u8>,
        action: Vec<u8>,
        reply: GoalReply,
        via_peer: Option<usize>,
    ) {
        let _ = self.try_insert_full(
            goal_id,
            client_identity,
            worker_identity,
            action,
            reply,
            via_peer,
            MAX_GOALS,
        );
    }

    pub fn remove(&mut self, goal_id: &[u8]) -> Option<GoalEntry> {
        self.goals.remove(goal_id)
    }

    pub fn get(&self, goal_id: &[u8]) -> Option<&GoalEntry> {
        self.goals.get(goal_id)
    }

    /// Drop all goals owned by `worker_identity`, returning them so the broker
    /// can synthesize WORKER_DIED results back to each client.
    pub fn evict_worker(&mut self, worker_identity: &[u8]) -> Vec<GoalEntry> {
        self.drain_if(|e| e.worker_identity == worker_identity)
    }

    /// Drop all goals forwarded via `peer_idx`.
    pub(crate) fn evict_peer(&mut self, peer_idx: usize) -> Vec<GoalEntry> {
        self.drain_if(|e| e.via_peer == Some(peer_idx))
    }

    /// Drop goals matching `pred`, returning them for reclaim.
    pub(crate) fn drain_if(&mut self, mut pred: impl FnMut(&GoalEntry) -> bool) -> Vec<GoalEntry> {
        let ids: Vec<Vec<u8>> = self
            .goals
            .iter()
            .filter(|(_, e)| pred(e))
            .map(|(k, _)| k.clone())
            .collect();
        let mut dropped = Vec::with_capacity(ids.len());
        for gid in ids {
            if let Some(e) = self.goals.remove(&gid) {
                dropped.push(e);
            }
        }
        dropped
    }
}

pub(crate) const HOP_SEP: char = ',';

pub(crate) fn hop_contains(hops: &str, broker_id: &str) -> bool {
    if broker_id.is_empty() {
        return false;
    }
    hops.split(HOP_SEP).any(|h| h == broker_id)
}

pub(crate) fn extend_hops(hops: &str, broker_id: &str) -> String {
    if hops.is_empty() {
        broker_id.to_string()
    } else {
        format!("{hops}{HOP_SEP}{broker_id}")
    }
}
