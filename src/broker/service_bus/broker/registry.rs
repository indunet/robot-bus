//! Worker registry for the service bus.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::inflight::{extend_hops, hop_contains};

/// Where a registered worker route comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerSource {
    /// DEALER connected to this broker's backend (real local worker).
    Local,
    /// Peer federation DEALER registered on this backend.
    Federated {
        /// Peer broker id this route was learned from.
        via_broker_id: String,
        /// Broker that hosts the real worker.
        origin_broker_id: String,
        /// Hop path at advertisement time (comma-separated broker ids).
        hop_path: String,
    },
}

#[derive(Clone, Debug)]
struct WorkerInfo {
    identity: Vec<u8>,
    last_heartbeat: Instant,
    in_flight: usize,
    source: WorkerSource,
}

pub struct WorkerRegistry {
    /// service_name -> workers (round-robin load-balanced)
    workers: HashMap<String, Vec<WorkerInfo>>,
    /// worker identity -> set of service names (federated peers may advertise many)
    by_identity: HashMap<Vec<u8>, std::collections::HashSet<String>>,
    /// Per-identity heartbeat (shared across services for that identity).
    identity_heartbeat: HashMap<Vec<u8>, Instant>,
    /// service_name -> next round-robin index (separate for local vs federated)
    rr_cursor_local: HashMap<String, usize>,
    rr_cursor_fed: HashMap<String, usize>,
}

impl WorkerRegistry {
    pub fn new() -> Self {
        Self {
            workers: HashMap::new(),
            by_identity: HashMap::new(),
            identity_heartbeat: HashMap::new(),
            rr_cursor_local: HashMap::new(),
            rr_cursor_fed: HashMap::new(),
        }
    }

    /// Register a local worker for a service. Re-registration moves the worker
    /// to the new service (old binding is dropped first).
    pub fn register(&mut self, identity: Vec<u8>, service: String, now: Instant) {
        // Local workers are 1:1 identity→service; drop any prior bindings.
        self.remove(&identity);
        self.insert_binding(identity, service, now, WorkerSource::Local);
    }

    /// Add or refresh a federated service binding without clearing other services
    /// on the same peer identity.
    pub fn register_federated(
        &mut self,
        identity: Vec<u8>,
        service: String,
        now: Instant,
        via_broker_id: String,
        origin_broker_id: String,
        hop_path: String,
    ) {
        // Replace this service binding for the identity if present.
        self.remove_service_binding(&identity, &service);
        self.insert_binding(
            identity,
            service,
            now,
            WorkerSource::Federated {
                via_broker_id,
                origin_broker_id,
                hop_path,
            },
        );
    }

    fn insert_binding(
        &mut self,
        identity: Vec<u8>,
        service: String,
        now: Instant,
        source: WorkerSource,
    ) {
        self.by_identity
            .entry(identity.clone())
            .or_default()
            .insert(service.clone());
        self.identity_heartbeat.insert(identity.clone(), now);
        self.workers.entry(service).or_default().push(WorkerInfo {
            identity,
            last_heartbeat: now,
            in_flight: 0,
            source,
        });
    }

    /// Refresh a worker's heartbeat timestamp (all service bindings).
    pub fn heartbeat(&mut self, identity: &[u8], now: Instant) {
        self.identity_heartbeat.insert(identity.to_vec(), now);
        if let Some(svcs) = self.by_identity.get(identity).cloned() {
            for svc in svcs {
                if let Some(list) = self.workers.get_mut(&svc) {
                    if let Some(w) = list.iter_mut().find(|w| w.identity == identity) {
                        w.last_heartbeat = now;
                    }
                }
            }
        }
    }

    /// Remove a worker from the registry entirely (all services).
    pub fn remove(&mut self, identity: &[u8]) {
        let Some(svcs) = self.by_identity.remove(identity) else {
            return;
        };
        self.identity_heartbeat.remove(identity);
        for svc in svcs {
            self.remove_from_service_list(&svc, identity);
        }
    }

    /// Remove one service binding for an identity; drop identity if none remain.
    pub fn remove_service_binding(&mut self, identity: &[u8], service: &str) {
        self.remove_from_service_list(service, identity);
        if let Some(svcs) = self.by_identity.get_mut(identity) {
            svcs.remove(service);
            if svcs.is_empty() {
                self.by_identity.remove(identity);
                self.identity_heartbeat.remove(identity);
            }
        }
    }

    fn remove_from_service_list(&mut self, service: &str, identity: &[u8]) {
        if let Some(list) = self.workers.get_mut(service) {
            list.retain(|w| w.identity != identity);
            if list.is_empty() {
                self.workers.remove(service);
                self.rr_cursor_local.remove(service);
                self.rr_cursor_fed.remove(service);
            }
        }
    }

    /// Evict workers whose last heartbeat is older than `timeout`.
    /// Returns the removed identities.
    pub fn sweep_dead(&mut self, now: Instant, timeout: Duration) -> Vec<Vec<u8>> {
        let dead: Vec<Vec<u8>> = self
            .identity_heartbeat
            .iter()
            .filter(|(_, hb)| now.duration_since(**hb) > timeout)
            .map(|(id, _)| id.clone())
            .collect();
        for identity in &dead {
            self.remove(identity);
        }
        dead
    }

    /// Pick the next worker for a service (round-robin) and bump its in-flight count.
    /// Prefers local workers: if any local worker exists, federated routes are ignored.
    pub fn select_worker(&mut self, service: &str) -> Option<Vec<u8>> {
        self.select_worker_filtered(service, |_| true)
    }

    /// Like [`select_worker`], but skips federated routes whose via/origin appear in `hop_path`.
    pub fn select_worker_avoiding_hops(
        &mut self,
        service: &str,
        hop_path: &str,
    ) -> Option<Vec<u8>> {
        self.select_worker_filtered(service, |w| match &w.source {
            WorkerSource::Local => true,
            WorkerSource::Federated {
                via_broker_id,
                origin_broker_id,
                ..
            } => {
                !hop_contains(hop_path, via_broker_id) && !hop_contains(hop_path, origin_broker_id)
            }
        })
    }

    fn select_worker_filtered(
        &mut self,
        service: &str,
        pred: impl Fn(&WorkerInfo) -> bool,
    ) -> Option<Vec<u8>> {
        let list = self.workers.get(service)?;
        let local_idxs: Vec<usize> = list
            .iter()
            .enumerate()
            .filter(|(_, w)| matches!(w.source, WorkerSource::Local) && pred(w))
            .map(|(i, _)| i)
            .collect();
        let pool: Vec<usize> = if !local_idxs.is_empty() {
            local_idxs
        } else {
            list.iter()
                .enumerate()
                .filter(|(_, w)| pred(w))
                .map(|(i, _)| i)
                .collect()
        };
        if pool.is_empty() {
            return None;
        }
        let use_local = matches!(list[pool[0]].source, WorkerSource::Local);
        let cursor_map = if use_local {
            &mut self.rr_cursor_local
        } else {
            &mut self.rr_cursor_fed
        };
        let cursor = cursor_map.entry(service.to_string()).or_insert(0);
        let pick = pool[*cursor % pool.len()];
        *cursor = (*cursor + 1) % pool.len();
        let list = self.workers.get_mut(service)?;
        list[pick].in_flight += 1;
        Some(list[pick].identity.clone())
    }

    /// Decrement a worker's in-flight count (called when its response arrives).
    pub fn release_worker(&mut self, identity: &[u8]) {
        if let Some(svcs) = self.by_identity.get(identity).cloned() {
            for svc in svcs {
                if let Some(list) = self.workers.get_mut(&svc) {
                    if let Some(w) = list.iter_mut().find(|w| w.identity == identity) {
                        if w.in_flight > 0 {
                            w.in_flight -= 1;
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Number of workers registered for a service.
    pub fn worker_count(&self, service: &str) -> usize {
        self.workers.get(service).map(Vec::len).unwrap_or(0)
    }

    /// Number of local workers registered for a service.
    pub fn local_worker_count(&self, service: &str) -> usize {
        self.workers
            .get(service)
            .map(|list| {
                list.iter()
                    .filter(|w| matches!(w.source, WorkerSource::Local))
                    .count()
            })
            .unwrap_or(0)
    }

    /// Whether any local worker is registered for `service`.
    pub fn has_local(&self, service: &str) -> bool {
        self.local_worker_count(service) > 0
    }

    /// Snapshot of offerings suitable for advertising to peers.
    /// Each entry is `(service, origin_broker_id, hop_path, via_broker_id_or_empty)`.
    /// Local workers use `via=""`, `origin=self_id`, `hop=self_id`.
    pub fn advertise_snapshot(&self, self_id: &str) -> Vec<(String, String, String, String)> {
        let mut out = Vec::new();
        for (svc, list) in &self.workers {
            for w in list {
                match &w.source {
                    WorkerSource::Local => {
                        out.push((
                            svc.clone(),
                            self_id.to_string(),
                            self_id.to_string(),
                            String::new(),
                        ));
                    }
                    WorkerSource::Federated {
                        via_broker_id,
                        origin_broker_id,
                        hop_path,
                    } => {
                        out.push((
                            svc.clone(),
                            origin_broker_id.clone(),
                            extend_hops(hop_path, self_id),
                            via_broker_id.clone(),
                        ));
                    }
                }
            }
        }
        out
    }

    /// Federated route metadata for a service binding, if any.
    pub fn federated_meta(
        &self,
        identity: &[u8],
        service: &str,
    ) -> Option<(String, String, String)> {
        let list = self.workers.get(service)?;
        let w = list.iter().find(|w| w.identity == identity)?;
        match &w.source {
            WorkerSource::Federated {
                via_broker_id,
                origin_broker_id,
                hop_path,
            } => Some((
                via_broker_id.clone(),
                origin_broker_id.clone(),
                hop_path.clone(),
            )),
            WorkerSource::Local => None,
        }
    }

    /// Look up the source for a registered identity on a given service.
    pub fn source_of(&self, identity: &[u8], service: &str) -> Option<&WorkerSource> {
        self.workers
            .get(service)?
            .iter()
            .find(|w| w.identity == identity)
            .map(|w| &w.source)
    }

    /// Whether a worker identity is currently registered.
    pub fn is_alive(&self, identity: &[u8]) -> bool {
        self.by_identity.contains_key(identity)
    }

    /// Service names bound to an identity.
    pub fn services_of(&self, identity: &[u8]) -> Vec<String> {
        self.by_identity
            .get(identity)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}
