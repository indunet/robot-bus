//! Best-effort HTTP registration of pub/sub endpoints with the broker console topology.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde_json::json;

use super::topic_type_register::resolve_console_url;

const REFRESH_INTERVAL: Duration = Duration::from_secs(10);

/// Keeps a topology endpoint alive until dropped (refresh + unregister).
pub struct TopologyEndpointGuard {
    endpoint_id: String,
    console_url: Option<String>,
    stop: Arc<AtomicBool>,
}

impl TopologyEndpointGuard {
    /// Register immediately and start a keep-alive refresh thread.
    pub fn start(
        console_url: Option<&str>,
        node_name: &str,
        kind: &str,
        topic: &str,
    ) -> Arc<Self> {
        let endpoint_id = uuid::Uuid::new_v4().to_string();
        let console_owned = console_url.map(|s| s.to_string());
        post_register(
            console_owned.as_deref(),
            &endpoint_id,
            node_name,
            kind,
            topic,
        );

        let stop = Arc::new(AtomicBool::new(false));
        let guard = Arc::new(Self {
            endpoint_id: endpoint_id.clone(),
            console_url: console_owned.clone(),
            stop: Arc::clone(&stop),
        });

        let refresh_url = console_owned;
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
                        refresh_url.as_deref(),
                        &refresh_id,
                        &refresh_node,
                        &refresh_kind,
                        &refresh_topic,
                    );
                }
            })
            .ok();

        guard
    }
}

impl Drop for TopologyEndpointGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        post_unregister(self.console_url.as_deref(), &self.endpoint_id);
    }
}

fn post_register(
    console_url: Option<&str>,
    endpoint_id: &str,
    node_name: &str,
    kind: &str,
    topic: &str,
) {
    let base = resolve_console_url(console_url);
    let url = format!("{base}/api/v1/topology/register");
    let body = json!({
        "endpointId": endpoint_id,
        "nodeName": node_name,
        "kind": kind,
        "topic": topic,
    });
    match ureq::post(&url).send_json(body) {
        Ok(resp) => {
            let status = resp.status();
            if !(200..300).contains(&status) {
                log::warn!(
                    "topology register {endpoint_id} ({kind} {topic}): HTTP {status} from {url}"
                );
            }
        }
        Err(err) => {
            log::warn!("topology register {endpoint_id} ({kind} {topic}) failed ({url}): {err}");
        }
    }
}

fn post_unregister(console_url: Option<&str>, endpoint_id: &str) {
    let base = resolve_console_url(console_url);
    let url = format!("{base}/api/v1/topology/unregister");
    let body = json!({
        "endpointId": endpoint_id,
    });
    match ureq::post(&url).send_json(body) {
        Ok(resp) => {
            let status = resp.status();
            if !(200..300).contains(&status) {
                log::warn!("topology unregister {endpoint_id}: HTTP {status} from {url}");
            }
        }
        Err(err) => {
            log::warn!("topology unregister {endpoint_id} failed ({url}): {err}");
        }
    }
}
