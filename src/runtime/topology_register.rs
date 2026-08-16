//! Best-effort topology endpoint registration on the broker control plane.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use prost::Message;

use crate::console_topics;
use crate::robot_bus_interfaces::msg::v1::{TopologyRegister, TopologyUnregister};
use crate::service_bus::ServiceClient;
use crate::transports;

const REFRESH_INTERVAL: Duration = Duration::from_secs(10);

/// Keeps a topology endpoint alive until dropped (refresh + unregister).
pub struct TopologyEndpointGuard {
    endpoint_id: String,
    service_frontend: String,
    stop: Arc<AtomicBool>,
}

impl TopologyEndpointGuard {
    /// Register immediately and start a keep-alive refresh thread.
    pub fn start(
        service_frontend: Option<&str>,
        host: &str,
        transport: &str,
        node_name: &str,
        kind: &str,
        topic: &str,
    ) -> Arc<Self> {
        let endpoint_id = uuid::Uuid::new_v4().to_string();
        let frontend = resolve_service_frontend(service_frontend, host, transport);
        if let Some(ref ep) = frontend {
            post_register(ep, &endpoint_id, node_name, kind, topic);
        }

        let stop = Arc::new(AtomicBool::new(false));
        let guard = Arc::new(Self {
            endpoint_id: endpoint_id.clone(),
            service_frontend: frontend.clone().unwrap_or_default(),
            stop: Arc::clone(&stop),
        });

        if let Some(refresh_frontend) = frontend {
            let refresh_id = endpoint_id;
            let refresh_node = node_name.to_string();
            let refresh_kind = kind.to_string();
            let refresh_topic = topic.to_string();
            let refresh_stop = Arc::clone(&stop);
            thread::Builder::new()
                .name("rbus-topo-refresh".into())
                .spawn(move || {
                    while !refresh_stop.load(Ordering::Relaxed) {
                        thread::sleep(REFRESH_INTERVAL);
                        if refresh_stop.load(Ordering::Relaxed) {
                            break;
                        }
                        post_register(
                            &refresh_frontend,
                            &refresh_id,
                            &refresh_node,
                            &refresh_kind,
                            &refresh_topic,
                        );
                    }
                })
                .ok();
        }

        guard
    }
}

impl Drop for TopologyEndpointGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if !self.service_frontend.is_empty() {
            post_unregister(&self.service_frontend, &self.endpoint_id);
        }
    }
}

fn resolve_service_frontend(explicit: Option<&str>, host: &str, transport: &str) -> Option<String> {
    if let Some(ep) = explicit {
        let t = ep.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if transport == "ws" {
        return transports::service_frontend_endpoint("127.0.0.1", "tcp").ok();
    }
    transports::service_frontend_endpoint(host, transport).ok()
}

fn post_register(
    service_frontend: &str,
    endpoint_id: &str,
    node_name: &str,
    kind: &str,
    topic: &str,
) {
    let payload = TopologyRegister {
        endpoint_id: endpoint_id.to_string(),
        node_name: node_name.to_string(),
        kind: kind.to_string(),
        topic: topic.to_string(),
    }
    .encode_to_vec();
    match ServiceClient::new(Some(service_frontend)).and_then(|client| {
        client.call(
            console_topics::TOPOLOGY_REGISTER,
            &payload,
            None,
            Some(Duration::from_secs(2)),
        )
    }) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("topology register {endpoint_id} ({kind} {topic}): {err}");
        }
    }
}

fn post_unregister(service_frontend: &str, endpoint_id: &str) {
    let payload = TopologyUnregister {
        endpoint_id: endpoint_id.to_string(),
    }
    .encode_to_vec();
    match ServiceClient::new(Some(service_frontend)).and_then(|client| {
        client.call(
            console_topics::TOPOLOGY_UNREGISTER,
            &payload,
            None,
            Some(Duration::from_secs(2)),
        )
    }) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("topology unregister {endpoint_id}: {err}");
        }
    }
}
