//! In-flight request table and hop-path helpers.

use std::collections::HashMap;
use std::time::Instant;

const HOP_SEP: char = ',';

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

/// A request waiting for an available worker.
pub(super) struct PendingRequest {
    pub(super) client_identity: Vec<u8>,
    pub(super) service: Vec<u8>,
    pub(super) request_id: Vec<u8>,
    pub(super) body: Vec<u8>,
    pub(super) has_req_delim: bool,
    pub(super) queued_at: Instant,
}

/// Tracks a request that has been forwarded to a worker and is awaiting a reply.
#[derive(Clone, Debug)]
pub struct InFlightEntry {
    pub client_identity: Vec<u8>,
    pub worker_identity: Vec<u8>,
    pub service: Vec<u8>,
    pub request_id: Vec<u8>,
    pub has_req_delim: bool,
}

/// In-flight RPC bookkeeping: keyed by `client_id\\0request_id`.
#[derive(Default)]
pub struct InFlightTable {
    entries: HashMap<Vec<u8>, InFlightEntry>,
}

impl InFlightTable {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(client_id: &[u8], request_id: &[u8]) -> Vec<u8> {
        let mut k = Vec::with_capacity(client_id.len() + 1 + request_id.len());
        k.extend_from_slice(client_id);
        k.push(0);
        k.extend_from_slice(request_id);
        k
    }

    pub fn insert(&mut self, entry: InFlightEntry) {
        let key = Self::key(&entry.client_identity, &entry.request_id);
        self.entries.insert(key, entry);
    }

    pub fn remove(&mut self, client_id: &[u8], request_id: &[u8]) -> Option<InFlightEntry> {
        self.entries.remove(&Self::key(client_id, request_id))
    }

    /// Drop all in-flight requests owned by `worker_identity`.
    pub fn evict_worker(&mut self, worker_identity: &[u8]) -> Vec<InFlightEntry> {
        let keys: Vec<Vec<u8>> = self
            .entries
            .iter()
            .filter(|(_, e)| e.worker_identity == worker_identity)
            .map(|(k, _)| k.clone())
            .collect();
        let mut dropped = Vec::with_capacity(keys.len());
        for k in keys {
            if let Some(e) = self.entries.remove(&k) {
                dropped.push(e);
            }
        }
        dropped
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
