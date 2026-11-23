//! Best-effort topology endpoint registration on the broker control plane.
//!
//! The first register and the final unregister run on helper threads. Callers
//! such as `create_publisher` must not wait out the RPC: with the console
//! disabled there is no worker, and the client would otherwise block until its
//! own timeout (the broker holds a worker-less request until `pending_timeout`).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use prost::Message;
use zmq::Context as ZmqContext;

use crate::console_topics;
use crate::robot_bus_interfaces::msg::v1::{TopologyRegister, TopologyUnregister};
use crate::service_bus::ServiceClient;
use crate::transports;
use crate::zmq_helpers::HighWaterMark;

const REFRESH_INTERVAL: Duration = Duration::from_secs(10);
const CALL_TIMEOUT: Duration = Duration::from_secs(2);

/// Keeps a topology endpoint alive until dropped (refresh + unregister).
pub struct TopologyEndpointGuard {
    endpoint_id: String,
    service_frontend: String,
    node_name: String,
    kind: String,
    topic: String,
    stop: Arc<AtomicBool>,
    /// Same context as the node so `inproc://` reaches the embedded broker.
    zmq: ZmqContext,
}

impl TopologyEndpointGuard {
    /// Register in the background and start a keep-alive refresh thread.
    pub fn start(
        zmq: &ZmqContext,
        service_frontend: Option<&str>,
        host: &str,
        transport: &str,
        node_name: &str,
        kind: &str,
        topic: &str,
    ) -> Arc<Self> {
        let endpoint_id = uuid::Uuid::new_v4().to_string();
        let frontend = resolve_service_frontend(service_frontend, host, transport);
        let stop = Arc::new(AtomicBool::new(false));
        let guard = Arc::new(Self {
            endpoint_id: endpoint_id.clone(),
            service_frontend: frontend.clone().unwrap_or_default(),
            node_name: node_name.to_string(),
            kind: kind.to_string(),
            topic: topic.to_string(),
            stop: Arc::clone(&stop),
            zmq: zmq.clone(),
        });

        if let Some(ep) = frontend.clone() {
            let zmq = zmq.clone();
            let id = endpoint_id.clone();
            let node = node_name.to_string();
            let kind_owned = kind.to_string();
            let topic_owned = topic.to_string();
            thread::Builder::new()
                .name("rbus-topo-reg".into())
                .spawn(move || post_register(&zmq, &ep, &id, &node, &kind_owned, &topic_owned))
                .ok();
        }

        if let Some(refresh_frontend) = frontend {
            let refresh_zmq = zmq.clone();
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
                            &refresh_zmq,
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

    /// Re-post this endpoint to the broker (used after a broker restart).
    pub(crate) fn refresh(&self) {
        if self.service_frontend.is_empty() || self.stop.load(Ordering::Relaxed) {
            return;
        }
        post_register(
            &self.zmq,
            &self.service_frontend,
            &self.endpoint_id,
            &self.node_name,
            &self.kind,
            &self.topic,
        );
    }
}

impl Drop for TopologyEndpointGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if self.service_frontend.is_empty() {
            return;
        }
        let zmq = self.zmq.clone();
        let endpoint = self.service_frontend.clone();
        let endpoint_id = self.endpoint_id.clone();
        thread::Builder::new()
            .name("rbus-topo-unreg".into())
            .spawn(move || post_unregister(&zmq, &endpoint, &endpoint_id))
            .ok();
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
    zmq: &ZmqContext,
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
    match ServiceClient::with_context_hwm(zmq, Some(service_frontend), HighWaterMark::RPC)
        .and_then(|client| {
            client.call(
                console_topics::TOPOLOGY_REGISTER,
                &payload,
                None,
                Some(CALL_TIMEOUT),
            )
        }) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("topology register {endpoint_id} ({kind} {topic}): {err}");
        }
    }
}

fn post_unregister(zmq: &ZmqContext, service_frontend: &str, endpoint_id: &str) {
    let payload = TopologyUnregister {
        endpoint_id: endpoint_id.to_string(),
    }
    .encode_to_vec();
    match ServiceClient::with_context_hwm(zmq, Some(service_frontend), HighWaterMark::RPC)
        .and_then(|client| {
            client.call(
                console_topics::TOPOLOGY_UNREGISTER,
                &payload,
                None,
                Some(CALL_TIMEOUT),
            )
        }) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("topology unregister {endpoint_id}: {err}");
        }
    }
}
