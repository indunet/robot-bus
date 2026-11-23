//! Best-effort endpoint registry for console topology (control plane only).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How long an endpoint may sit without refresh before expiry.
pub const ENDPOINT_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointKind {
    Publisher,
    Subscriber,
    ServiceClient,
    ServiceServer,
    ActionClient,
    ActionServer,
}

impl EndpointKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Publisher => "publisher",
            Self::Subscriber => "subscriber",
            Self::ServiceClient => "service_client",
            Self::ServiceServer => "service_server",
            Self::ActionClient => "action_client",
            Self::ActionServer => "action_server",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "publisher" | "pub" => Some(Self::Publisher),
            "subscriber" | "sub" => Some(Self::Subscriber),
            "service_client" | "svc_client" => Some(Self::ServiceClient),
            "service_server" | "svc_server" => Some(Self::ServiceServer),
            "action_client" | "act_client" => Some(Self::ActionClient),
            "action_server" | "act_server" => Some(Self::ActionServer),
            _ => None,
        }
    }

    /// Intermediate hub in the snapshot: `topic` / `service` / `action`.
    pub fn hub_kind(self) -> &'static str {
        match self {
            Self::Publisher | Self::Subscriber => "topic",
            Self::ServiceClient | Self::ServiceServer => "service",
            Self::ActionClient | Self::ActionServer => "action",
        }
    }

    /// True when the process node is the snapshot-edge source (emits toward the hub).
    pub fn process_is_source(self) -> bool {
        matches!(
            self,
            Self::Publisher | Self::ServiceClient | Self::ActionClient
        )
    }

    pub fn is_pubsub(self) -> bool {
        matches!(self, Self::Publisher | Self::Subscriber)
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

#[derive(Debug, Clone)]
pub struct TopologyGraphNode {
    pub id: String,
    pub kind: &'static str,
    pub label: String,
    pub type_name: Option<String>,
    pub msg_per_sec: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct TopologyGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: &'static str,
    pub topic: String,
}

#[derive(Debug, Clone)]
pub struct TopologyGraph {
    pub nodes: Vec<TopologyGraphNode>,
    pub edges: Vec<TopologyGraphEdge>,
}

/// Build the console topology graph from live endpoints plus optional rates.
///
/// `topic_ops` / `service_ops` / `action_ops` are ops-per-second keyed by name
/// (topic `msg_per_sec`, service `calls_per_sec`, action `runs_per_sec`).
pub fn build_topology_graph(
    endpoints: &[EndpointRecord],
    type_map: &HashMap<String, String>,
    topic_ops: &HashMap<String, f64>,
    service_ops: &HashMap<String, f64>,
    action_ops: &HashMap<String, f64>,
) -> TopologyGraph {
    let mut process_names: HashSet<String> = HashSet::new();
    let mut topic_names: HashSet<String> = HashSet::new();
    let mut service_names: HashSet<String> = HashSet::new();
    let mut action_names: HashSet<String> = HashSet::new();
    let mut edges = Vec::with_capacity(endpoints.len());

    for ep in endpoints {
        process_names.insert(ep.node_name.clone());
        match ep.kind.hub_kind() {
            "service" => {
                service_names.insert(ep.topic.clone());
            }
            "action" => {
                action_names.insert(ep.topic.clone());
            }
            _ => {
                topic_names.insert(ep.topic.clone());
            }
        }
        let process_id = format!("node:{}", ep.node_name);
        let hub_id = format!("{}:{}", ep.kind.hub_kind(), ep.topic);
        let (source, target) = if ep.kind.process_is_source() {
            (process_id, hub_id)
        } else {
            (hub_id, process_id)
        };
        edges.push(TopologyGraphEdge {
            id: ep.endpoint_id.clone(),
            source,
            target,
            kind: ep.kind.as_str(),
            topic: ep.topic.clone(),
        });
    }

    for name in topic_ops.keys().chain(type_map.keys()) {
        topic_names.insert(name.clone());
    }
    for name in service_ops.keys() {
        service_names.insert(name.clone());
    }
    for name in action_ops.keys() {
        action_names.insert(name.clone());
    }

    let mut nodes = Vec::new();
    let mut process_sorted: Vec<_> = process_names.into_iter().collect();
    process_sorted.sort();
    for name in process_sorted {
        nodes.push(TopologyGraphNode {
            id: format!("node:{name}"),
            kind: "process",
            label: name,
            type_name: None,
            msg_per_sec: None,
        });
    }

    let mut topic_sorted: Vec<_> = topic_names.into_iter().collect();
    topic_sorted.sort();
    for name in topic_sorted {
        nodes.push(TopologyGraphNode {
            id: format!("topic:{name}"),
            kind: "topic",
            label: name.clone(),
            type_name: type_map.get(&name).cloned(),
            msg_per_sec: topic_ops.get(&name).copied(),
        });
    }

    let mut service_sorted: Vec<_> = service_names.into_iter().collect();
    service_sorted.sort();
    for name in service_sorted {
        nodes.push(TopologyGraphNode {
            id: format!("service:{name}"),
            kind: "service",
            label: name.clone(),
            type_name: None,
            msg_per_sec: service_ops.get(&name).copied(),
        });
    }

    let mut action_sorted: Vec<_> = action_names.into_iter().collect();
    action_sorted.sort();
    for name in action_sorted {
        nodes.push(TopologyGraphNode {
            id: format!("action:{name}"),
            kind: "action",
            label: name.clone(),
            type_name: None,
            msg_per_sec: action_ops.get(&name).copied(),
        });
    }

    TopologyGraph { nodes, edges }
}

/// Shared map of live endpoints registered by clients.
#[derive(Debug, Default)]
pub struct TopologyRegistry {
    endpoints: Mutex<HashMap<String, EndpointRecord>>,
}

impl TopologyRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Upsert an endpoint and refresh `last_seen`.
    pub fn register(&self, endpoint_id: &str, node_name: &str, kind: EndpointKind, topic: &str) {
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

    pub fn unregister(&self, endpoint_id: &str) -> Option<EndpointRecord> {
        let mut map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        map.remove(endpoint_id)
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
                _ => {}
            }
        }
        (pubs, subs)
    }

    /// Publisher/subscriber counts keyed by topic (after sweep).
    ///
    /// Service / action endpoints are ignored so Topics table counts stay pub/sub.
    pub fn counts_by_topic(&self) -> HashMap<String, (u64, u64)> {
        self.sweep_expired();
        let map = self.endpoints.lock().unwrap_or_else(|e| e.into_inner());
        let mut out: HashMap<String, (u64, u64)> = HashMap::new();
        for rec in map.values() {
            if !rec.kind.is_pubsub() {
                continue;
            }
            let entry = out.entry(rec.topic.clone()).or_insert((0, 0));
            match rec.kind {
                EndpointKind::Publisher => entry.0 += 1,
                EndpointKind::Subscriber => entry.1 += 1,
                _ => {}
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
        assert!(r.unregister("e1").is_some());
        assert_eq!(r.counts_for_topic("/imu"), (0, 1));
    }

    #[test]
    fn parse_kind() {
        assert_eq!(
            EndpointKind::parse("publisher"),
            Some(EndpointKind::Publisher)
        );
        assert_eq!(EndpointKind::parse("SUB"), Some(EndpointKind::Subscriber));
        assert_eq!(
            EndpointKind::parse("service_server"),
            Some(EndpointKind::ServiceServer)
        );
        assert_eq!(
            EndpointKind::parse("SVC_CLIENT"),
            Some(EndpointKind::ServiceClient)
        );
        assert_eq!(
            EndpointKind::parse("action_server"),
            Some(EndpointKind::ActionServer)
        );
        assert_eq!(
            EndpointKind::parse("act_client"),
            Some(EndpointKind::ActionClient)
        );
        assert_eq!(EndpointKind::parse("nope"), None);
    }

    #[test]
    fn counts_by_topic_ignores_service_and_action() {
        let r = TopologyRegistry::new();
        r.register("e1", "talker", EndpointKind::Publisher, "/imu");
        r.register("e2", "worker", EndpointKind::ServiceServer, "/set_bool");
        r.register("e3", "caller", EndpointKind::ServiceClient, "/set_bool");
        r.register("e4", "worker", EndpointKind::ActionServer, "/fibonacci");
        let counts = r.counts_by_topic();
        assert_eq!(counts.get("/imu"), Some(&(1, 0)));
        assert!(!counts.contains_key("/set_bool"));
        assert!(!counts.contains_key("/fibonacci"));
        assert_eq!(r.counts_for_topic("/set_bool"), (0, 0));
    }

    #[test]
    fn graph_wires_service_client_to_server() {
        let r = TopologyRegistry::new();
        r.register("e1", "caller", EndpointKind::ServiceClient, "/set_bool");
        r.register("e2", "worker", EndpointKind::ServiceServer, "/set_bool");
        let mut svc_ops = HashMap::new();
        svc_ops.insert("/set_bool".into(), 3.0);
        let g = build_topology_graph(
            &r.snapshot(),
            &HashMap::new(),
            &HashMap::new(),
            &svc_ops,
            &HashMap::new(),
        );
        assert!(
            g.nodes
                .iter()
                .any(|n| n.id == "node:caller" && n.kind == "process")
        );
        assert!(g.nodes.iter().any(|n| n.id == "node:worker"));
        let hub = g
            .nodes
            .iter()
            .find(|n| n.kind == "service" && n.label == "/set_bool")
            .expect("service hub");
        assert_eq!(hub.msg_per_sec, Some(3.0));
        let kinds: Vec<_> = g.edges.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&"service_client"));
        assert!(kinds.contains(&"service_server"));
        let client = g.edges.iter().find(|e| e.kind == "service_client").unwrap();
        assert_eq!(client.source, "node:caller");
        assert_eq!(client.target, "service:/set_bool");
        let server = g.edges.iter().find(|e| e.kind == "service_server").unwrap();
        assert_eq!(server.source, "service:/set_bool");
        assert_eq!(server.target, "node:worker");
    }
}
