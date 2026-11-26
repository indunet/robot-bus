//! HTTP discover client.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::config::{DEFAULT_API_DISCOVER_PATH, DiscoverOpts};
use super::packet::BrokerAnnouncement;
use crate::errors::{BusError, Result};
use crate::robot_bus_interfaces::msg::v1::TcpPorts;

/// JSON body of `GET /api/v1/discover`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverResponse {
    pub broker_id: String,
    #[serde(default)]
    pub domain_id: u32,
    /// Explicit TCP host override; otherwise clients use their discovery URL's host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advertise_host: Option<String>,
    pub message_xsub_port: u16,
    pub message_xpub_port: u16,
    pub service_frontend_port: u16,
    pub service_backend_port: u16,
    pub action_frontend_port: u16,
    pub action_backend_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipc_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inproc_prefix: Option<String>,
}

impl DiscoverResponse {
    /// Resolve a response using the API base the client originally contacted.
    pub fn to_announcement(&self, api_base: &str) -> Result<BrokerAnnouncement> {
        let base = normalize_api_base(api_base);
        let host = self.tcp_host_at(&base)?;
        self.validate_ports()?;
        Ok(BrokerAnnouncement {
            broker_id: self.broker_id.clone(),
            domain_id: self.domain_id,
            advertise_host: host,
            tcp: Some(TcpPorts {
                message_xsub: self.message_xsub_port.into(),
                message_xpub: self.message_xpub_port.into(),
                service_frontend: self.service_frontend_port.into(),
                service_backend: self.service_backend_port.into(),
                action_frontend: self.action_frontend_port.into(),
                action_backend: self.action_backend_port.into(),
            }),
            ipc_dir: self.ipc_dir.clone(),
            inproc_prefix: self.inproc_prefix.clone(),
            ws_url: Some(base.clone()),
            console_url: Some(base),
        })
    }

    pub(crate) fn tcp_host_at(&self, base: &str) -> Result<String> {
        if let Some(host) = &self.advertise_host {
            if let Ok(ip) = host.parse::<std::net::Ipv6Addr>() {
                return Ok(format!("[{ip}]"));
            }
            return Ok(host.clone());
        }
        let url = url::Url::parse(base).map_err(|err| {
            BusError::Protocol(format!("invalid broker discovery URL {base:?}: {err}"))
        })?;
        let host = url.host_str().ok_or_else(|| {
            BusError::Protocol(format!("broker discovery URL {base:?} has no host"))
        })?;
        Ok(host.to_owned())
    }

    pub(crate) fn validate_ports(&self) -> Result<()> {
        for (name, port) in [
            ("messageXsubPort", self.message_xsub_port),
            ("messageXpubPort", self.message_xpub_port),
            ("serviceFrontendPort", self.service_frontend_port),
            ("serviceBackendPort", self.service_backend_port),
            ("actionFrontendPort", self.action_frontend_port),
            ("actionBackendPort", self.action_backend_port),
        ] {
            if port == 0 {
                return Err(BusError::Protocol(format!(
                    "discover {name} has unresolved port 0"
                )));
            }
        }
        Ok(())
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
            "discover GET {url} failed: {e} (is the broker API listening? default --api-listen 0.0.0.0:15560)"
        ))
    })?;
    let disc = resp
        .into_json::<DiscoverResponse>()
        .map_err(|e| BusError::Protocol(format!("discover decode {url}: {e}")))?;
    disc.validate_ports()?;
    Ok(disc)
}

pub fn wait(opts: DiscoverOpts) -> Result<BrokerAnnouncement> {
    let base = normalize_api_base(&opts.api_url);
    let disc = fetch_discover(&base, opts.timeout)?;
    if let Some(want) = &opts.broker_id {
        if &disc.broker_id != want {
            return Err(BusError::Protocol(format!(
                "discover broker_id mismatch: got {:?}, want {want:?}",
                disc.broker_id
            )));
        }
    }
    disc.to_announcement(&base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeOptions;

    fn response() -> DiscoverResponse {
        serde_json::from_value(serde_json::json!({
            "brokerId": "broker",
            "messageXsubPort": 15580,
            "messageXpubPort": 15581,
            "serviceFrontendPort": 15662,
            "serviceBackendPort": 15663,
            "actionFrontendPort": 15664,
            "actionBackendPort": 15665
        }))
        .unwrap()
    }

    #[test]
    fn discovery_host_selection_uses_api_host_unless_overridden() {
        let mut disc = response();
        assert!(
            serde_json::to_value(&disc)
                .unwrap()
                .get("advertiseHost")
                .is_none()
        );
        for (override_host, wanted) in [(None, "robot.example"), (Some("192.0.2.1"), "192.0.2.1")] {
            disc.advertise_host = override_host.map(str::to_owned);
            let base = "https://robot.example:8443";
            let announcement = disc.to_announcement(base).unwrap();
            assert_eq!(announcement.advertise_host, wanted);
            assert_eq!(announcement.ws_url.as_deref(), Some(base));
            assert_eq!(announcement.console_url.as_deref(), Some(base));
        }
    }

    #[test]
    fn discovered_tcp_uses_ipv6_host_and_data_port() {
        let disc = response();
        let ann = disc.to_announcement("http://[2001:db8::2]:15560").unwrap();
        let opts = ann.apply(NodeOptions::tcp()).unwrap();
        assert_eq!(
            opts.message_xpub_endpoint().unwrap(),
            "tcp://[2001:db8::2]:15581"
        );
        assert_eq!(
            disc.tcp_host_at("http://localhost:15560").unwrap(),
            "localhost"
        );
    }

    #[test]
    fn discovered_tcp_rejects_unresolved_port() {
        let mut disc = response();
        disc.message_xpub_port = 0;
        let err = disc
            .to_announcement("http://robot.example:15560")
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("messageXpubPort has unresolved port 0")
        );
    }
}
