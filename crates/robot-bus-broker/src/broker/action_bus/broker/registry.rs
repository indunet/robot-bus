//! Worker registry for the action bus.

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct WorkerInfo {
    identity: Vec<u8>,
    last_heartbeat: Instant,
    in_flight: usize,
}

pub struct WorkerRegistry {
    /// action_name -> workers (round-robin load-balanced)
    workers: HashMap<String, Vec<WorkerInfo>>,
    /// worker identity -> action_name (reverse index for heartbeat/remove)
    by_identity: HashMap<Vec<u8>, String>,
    /// action_name -> next round-robin index
    rr_cursor: HashMap<String, usize>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            by_identity: HashMap::new(),
            rr_cursor: HashMap::new(),
        }
    }

    /// Register a worker for an action. Re-registration moves the worker to
    /// the new action (old binding is dropped first).
    pub fn register(&mut self, identity: Vec<u8>, action: String, now: Instant) {
        self.remove(&identity);
        self.by_identity.insert(identity.clone(), action.clone());
        self.workers.entry(action).or_default().push(WorkerInfo {
            identity,
            last_heartbeat: now,
            in_flight: 0,
        });
    }

    /// Refresh a worker's heartbeat timestamp.
    pub fn heartbeat(&mut self, identity: &[u8], now: Instant) {
        if let Some(act) = self.by_identity.get(identity).cloned() {
            if let Some(list) = self.workers.get_mut(&act) {
                if let Some(w) = list.iter_mut().find(|w| w.identity == identity) {
                    w.last_heartbeat = now;
                }
            }
        }
    }

    /// Remove a worker from the registry entirely. Returns the action it was bound to.
    pub fn remove(&mut self, identity: &[u8]) -> Option<String> {
        if let Some(act) = self.by_identity.remove(identity) {
            if let Some(list) = self.workers.get_mut(&act) {
                list.retain(|w| &w.identity != identity);
                if list.is_empty() {
                    self.workers.remove(&act);
                    self.rr_cursor.remove(&act);
                }
            }
            Some(act)
        } else {
            None
        }
    }

    /// Evict workers whose last heartbeat is older than `timeout`. Returns the
    /// identities of the evicted workers so the caller can reclaim their goals.
    pub fn sweep_dead(&mut self, now: Instant, timeout: Duration) -> Vec<(Vec<u8>, String)> {
        let dead: Vec<(Vec<u8>, String)> = self
            .workers
            .values()
            .flat_map(|list| {
                list.iter()
                    .filter(|w| now.duration_since(w.last_heartbeat) > timeout)
                    .filter_map(|w| {
                        self.by_identity
                            .get(&w.identity)
                            .cloned()
                            .map(|act| (w.identity.clone(), act))
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        for (identity, _) in &dead {
            self.remove(identity);
        }
        dead
    }

    /// Pick the next worker for an action (round-robin) and bump its in-flight count.
    pub fn select_worker(&mut self, action: &str) -> Option<Vec<u8>> {
        let list = self.workers.get_mut(action)?;
        if list.is_empty() {
            return None;
        }
        let cursor = self.rr_cursor.entry(action.to_string()).or_insert(0);
        let idx = *cursor % list.len();
        *cursor = (*cursor + 1) % list.len();
        list[idx].in_flight += 1;
        Some(list[idx].identity.clone())
    }

    /// Decrement a worker's in-flight count (called when its result arrives).
    pub fn release_worker(&mut self, identity: &[u8]) {
        if let Some(act) = self.by_identity.get(identity).cloned() {
            if let Some(list) = self.workers.get_mut(&act) {
                if let Some(w) = list.iter_mut().find(|w| &w.identity == identity) {
                    if w.in_flight > 0 {
                        w.in_flight -= 1;
                    }
                }
            }
        }
    }

    /// Number of workers registered for an action.
    pub fn worker_count(&self, action: &str) -> usize {
        self.workers.get(action).map(Vec::len).unwrap_or(0)
    }

    /// Snapshot of action names with at least one local worker.
    pub fn action_names(&self) -> Vec<String> {
        self.workers.keys().cloned().collect()
    }

    /// Whether a worker identity is currently registered.
    pub fn is_alive(&self, identity: &[u8]) -> bool {
        self.by_identity.contains_key(identity)
    }
}
