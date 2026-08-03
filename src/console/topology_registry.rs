//! Best-effort pub/sub endpoint registry for console topology (control plane only).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How long an endpoint may sit without refresh before expiry.
pub const ENDPOINT_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointKind {
    Publisher,
    Subscriber,
}

impl EndpointKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Publisher => "publisher",
            Self::Subscriber => "subscriber",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "publisher" | "pub" => Some(Self::Publisher),
            "subscriber" | "sub" => Some(Self::Subscriber),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EndpointRecord {
    pub endpoint_id: String,
    pub node_name: String,
    pub kind: EndpointKind,
    pub topic: String,
    pub last_seen: Instant,
    pub last_seen_unix_ms: u64,
}

/// Shared map of live pub/sub endpoints registered by clients.
#[derive(Debug, Default)]
pub struct TopologyRegistry {
    endpoints: Mutex<HashMap<String, EndpointRecord>>,
}

impl TopologyRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Upsert an endpoint and refresh `last_seen`.
    pub fn register(
        &self,
        endpoint_id: &str,
        node_name: &str,
        kind: EndpointKind,
        topic: &str,
    ) {
        let now = Instant::now();
        let mut map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(
            endpoint_id.to_string(),
            EndpointRecord {
                endpoint_id: endpoint_id.to_string(),
                node_name: node_name.to_string(),
                kind,
                topic: topic.to_string(),
                last_seen: now,
                last_seen_unix_ms: unix_ms(),
            },
        );
    }

    pub fn unregister(&self, endpoint_id: &str) -> bool {
        let mut map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        map.remove(endpoint_id).is_some()
    }

    /// Drop endpoints that have not refreshed within [`ENDPOINT_TTL`].
    pub fn sweep_expired(&self) -> usize {
        let mut map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        let before = map.len();
        map.retain(|_, rec| rec.last_seen.elapsed() < ENDPOINT_TTL);
        before.saturating_sub(map.len())
    }

    /// Live endpoints after TTL sweep, sorted by topic then node.
    pub fn snapshot(&self) -> Vec<EndpointRecord> {
        self.sweep_expired();
        let map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        let mut out: Vec<_> = map.values().cloned().collect();
        out.sort_by(|a, b| {
            a.topic
                .cmp(&b.topic)
                .then(a.node_name.cmp(&b.node_name))
                .then(a.endpoint_id.cmp(&b.endpoint_id))
        });
        out
    }

    /// Count live publishers and subscribers for a topic (after sweep).
    pub fn counts_for_topic(&self, topic: &str) -> (u64, u64) {
        self.sweep_expired();
        let map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        let mut pubs = 0u64;
        let mut subs = 0u64;
        for rec in map.values() {
            if rec.topic != topic {
                continue;
            }
            match rec.kind {
                EndpointKind::Publisher => pubs += 1,
                EndpointKind::Subscriber => subs += 1,
            }
        }
        (pubs, subs)
    }

    /// Publisher/subscriber counts keyed by topic (after sweep).
    pub fn counts_by_topic(&self) -> HashMap<String, (u64, u64)> {
        self.sweep_expired();
        let map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        let mut out: HashMap<String, (u64, u64)> = HashMap::new();
        for rec in map.values() {
            let entry = out.entry(rec.topic.clone()).or_insert((0, 0));
            match rec.kind {
                EndpointKind::Publisher => entry.0 += 1,
                EndpointKind::Subscriber => entry.1 += 1,
            }
        }
        out
    }
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_counts_and_unregister() {
        let r = TopologyRegistry::new();
        r.register("e1", "talker", EndpointKind::Publisher, "/imu");
        r.register("e2", "listener", EndpointKind::Subscriber, "/imu");
        assert_eq!(r.counts_for_topic("/imu"), (1, 1));
        assert!(r.unregister("e1"));
        assert_eq!(r.counts_for_topic("/imu"), (0, 1));
    }

    #[test]
    fn parse_kind() {
        assert_eq!(EndpointKind::parse("publisher"), Some(EndpointKind::Publisher));
        assert_eq!(EndpointKind::parse("SUB"), Some(EndpointKind::Subscriber));
        assert_eq!(EndpointKind::parse("nope"), None);
    }
}
