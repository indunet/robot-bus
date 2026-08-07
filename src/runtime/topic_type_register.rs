//! Best-effort registration of topic → type on the broker control plane.

use prost::Message;

use crate::console_topics;
use crate::robot_bus_interface::msg::v1::TopicTypeRegister;
use crate::service_bus::ServiceClient;
use crate::transports;

fn resolve_service_frontend(explicit: Option<&str>, host: &str, transport: &str) -> Option<String> {
    if let Some(ep) = explicit {
        let t = ep.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    if transport == "grpc" {
        return transports::service_frontend_endpoint("127.0.0.1", "tcp").ok();
    }
    transports::service_frontend_endpoint(host, transport).ok()
}

/// Register a topic type through the broker's reliable control-plane service.
pub fn register_topic_type(
    service_frontend: Option<&str>,
    host: &str,
    transport: &str,
    topic: &str,
    type_name: &str,
) {
    let payload = TopicTypeRegister {
        topic: topic.to_string(),
        type_name: type_name.to_string(),
    }
    .encode_to_vec();

    let Some(endpoint) = resolve_service_frontend(service_frontend, host, transport) else {
        return;
    };
    match ServiceClient::new(Some(&endpoint)).and_then(|client| {
        client.call(
            console_topics::TOPIC_TYPE_REGISTER,
            &payload,
            None,
            Some(std::time::Duration::from_secs(2)),
        )
    }) {
        Ok(_) => {}
        Err(err) => {
            log::warn!("topic type register {topic} -> {type_name} failed ({endpoint}): {err}");
        }
    }
}
