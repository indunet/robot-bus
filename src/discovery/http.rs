//! HTTP discover client.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::config::{DEFAULT_API_DISCOVER_PATH, DiscoverOpts};
use super::net::tcp_port_from_bind;
use super::packet::BrokerAnnouncement;
use crate::errors::{BusError, Result};
use crate::generated::robot_bus_interfaces::msg::v1::TcpPorts;

/// JSON body of `GET /api/v1/discover`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverResponse {
    pub broker_id: String,
    #[serde(default)]
    pub domain_id: u32,
    pub advertise_host: String,
    pub api_url: String,
    pub message_xsub: String,
    pub message_xpub: String,
    pub service_frontend: String,
    pub service_backend: String,
    pub action_frontend: String,
    pub action_backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipc_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inproc_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub console_url: Option<String>,
}

impl DiscoverResponse {
    pub fn to_announcement(&self) -> Result<BrokerAnnouncement> {
        let tcp = TcpPorts {
            message_xsub: require_port(&self.message_xsub, "messageXsub")?,
            message_xpub: require_port(&self.message_xpub, "messageXpub")?,
            service_frontend: require_port(&self.service_frontend, "serviceFrontend")?,
            service_backend: require_port(&self.service_backend, "serviceBackend")?,
            action_frontend: require_port(&self.action_frontend, "actionFrontend")?,
            action_backend: require_port(&self.action_backend, "actionBackend")?,
        };
        Ok(BrokerAnnouncement {
            broker_id: self.broker_id.clone(),
            domain_id: self.domain_id,
            advertise_host: self.advertise_host.clone(),
            tcp: Some(tcp),
            ipc_dir: self.ipc_dir.clone(),
            inproc_prefix: self.inproc_prefix.clone(),
            ws_url: Some(self.api_url.clone()),
            console_url: self
                .console_url
                .clone()
                .or_else(|| Some(self.api_url.clone())),
        })
    }
}

/// Normalize `host:port`, `http://host:port`, or trailing-slash bases to `http://host:port`.
pub fn normalize_api_base(input: &str) -> String {
    let s = input.trim().trim_end_matches('/');
    if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("http://{s}")
    }
}

/// Rewrite `tcp://0.0.0.0:port` / `tcp://*:port` to `tcp://{host}:port`.
pub fn rewrite_bind_host(bind: &str, host: &str) -> String {
    if let Some(rest) = bind.strip_prefix("tcp://0.0.0.0:") {
        return format!("tcp://{host}:{rest}");
    }
    if let Some(rest) = bind.strip_prefix("tcp://*:") {
        return format!("tcp://{host}:{rest}");
    }
    if let Some(rest) = bind.strip_prefix("tcp://[::]:") {
        return format!("tcp://{host}:{rest}");
    }
    bind.to_string()
}

/// GET `/api/v1/discover` from `api_base`.
pub fn fetch_discover(api_base: &str, timeout: Duration) -> Result<DiscoverResponse> {
    let base = normalize_api_base(api_base);
    let url = format!("{base}{DEFAULT_API_DISCOVER_PATH}");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .build();
    let resp = agent.get(&url).call().map_err(|e| {
        BusError::Protocol(format!(
            "discover GET {url} failed: {e} (is the broker API listening? default --api-listen 0.0.0.0:15570)"
        ))
    })?;
    resp.into_json::<DiscoverResponse>()
        .map_err(|e| BusError::Protocol(format!("discover decode {url}: {e}")))
}

pub fn wait(opts: DiscoverOpts) -> Result<BrokerAnnouncement> {
    let disc = fetch_discover(&opts.api_url, opts.timeout)?;
    if let Some(want) = &opts.broker_id {
        if &disc.broker_id != want {
            return Err(BusError::Protocol(format!(
                "discover broker_id mismatch: got {:?}, want {want:?}",
                disc.broker_id
            )));
        }
    }
    disc.to_announcement()
}

fn require_port(endpoint: &str, name: &str) -> Result<u32> {
    let port = tcp_port_from_bind(endpoint).ok_or_else(|| {
        BusError::Protocol(format!("discover {name} missing tcp port: {endpoint}"))
    })?;
    if port == 0 {
        return Err(BusError::Protocol(format!(
            "discover {name} has unresolved port 0: {endpoint}"
        )));
    }
    Ok(u32::from(port))
}
