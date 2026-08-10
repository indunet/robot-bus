//! HTTP API broker discovery (control plane).
//!
//! Brokers expose [`GET /api/v1/discover`](crate::discovery::DiscoverResponse) on the
//! API listen port (default `15570`). Clients fetch that JSON, then
//! [`BrokerAnnouncement::apply`] fills location fields on a user-chosen
//! [`crate::NodeOptions`] transport (`tcp` / `ipc` / `inproc` / `grpc`).
//!
//! UDP multicast announce is deprecated and no longer started by the broker.

mod config;
mod http;
mod net;
mod packet;

#[allow(deprecated)]
pub use config::{
    DEFAULT_API_DISCOVER_PATH, DEFAULT_DISCOVERY_TIMEOUT, DEFAULT_MULTICAST_ADDR,
    DiscoverOpts, DiscoveryConfig, MAGIC, SCHEMA_VERSION,
};
#[allow(deprecated)]
pub use config::DEFAULT_DISCOVERY_PORT;
pub use http::{
    DiscoverResponse, fetch_discover, normalize_api_base, rewrite_bind_host,
};
pub use net::{infer_advertise_host, resolve_advertise_host, tcp_port_from_bind};
pub use packet::{BrokerAnnouncement, decode_announce, encode_announce, try_parse_datagram};

use crate::errors::{BusError, Result};
use crate::runtime::NodeOptions;
use crate::transports::{
    ACTION_BACKEND_CHANNEL, ACTION_FRONTEND_CHANNEL, SERVICE_BACKEND_CHANNEL,
    SERVICE_FRONTEND_CHANNEL, XPUB_CHANNEL, XSUB_CHANNEL, inproc_endpoint_with_prefix,
    ipc_endpoint_in, tcp_endpoint,
};

/// Client wait: HTTP GET `/api/v1/discover` on the broker API port.
pub fn wait(opts: DiscoverOpts) -> Result<BrokerAnnouncement> {
    http::wait(opts)
}

impl BrokerAnnouncement {
    /// Fill location fields on `opts` while keeping the user's chosen transport.
    ///
    /// Explicit endpoint fields already set on `opts` are left unchanged.
    pub fn apply(&self, mut opts: NodeOptions) -> Result<NodeOptions> {
        match opts.transport.as_str() {
            "tcp" => self.apply_tcp(&mut opts)?,
            "ipc" => self.apply_ipc(&mut opts)?,
            "inproc" => self.apply_inproc(&mut opts)?,
            "grpc" => self.apply_grpc(&mut opts)?,
            other => {
                return Err(BusError::Protocol(format!(
                    "discovery apply: unknown transport {other:?}"
                )));
            }
        }
        if opts.console_url.is_none() {
            if let Some(url) = self.console_url.clone() {
                opts.console_url = Some(url);
            }
        }
        Ok(opts)
    }

    fn apply_tcp(&self, opts: &mut NodeOptions) -> Result<()> {
        let tcp = self
            .tcp
            .as_ref()
            .ok_or_else(|| BusError::Protocol("discovery announce missing tcp ports".into()))?;
        opts.host = self.advertise_host.clone();
        if opts.message_xsub.is_none() {
            opts.message_xsub = Some(tcp_endpoint(
                &self.advertise_host,
                port_u16(tcp.message_xsub, "message_xsub")?,
            ));
        }
        if opts.message_xpub.is_none() {
            opts.message_xpub = Some(tcp_endpoint(
                &self.advertise_host,
                port_u16(tcp.message_xpub, "message_xpub")?,
            ));
        }
        if opts.service_frontend.is_none() {
            opts.service_frontend = Some(tcp_endpoint(
                &self.advertise_host,
                port_u16(tcp.service_frontend, "service_frontend")?,
            ));
        }
        if opts.service_backend.is_none() {
            opts.service_backend = Some(tcp_endpoint(
                &self.advertise_host,
                port_u16(tcp.service_backend, "service_backend")?,
            ));
        }
        if opts.action_frontend.is_none() {
            opts.action_frontend = Some(tcp_endpoint(
                &self.advertise_host,
                port_u16(tcp.action_frontend, "action_frontend")?,
            ));
        }
        if opts.action_backend.is_none() {
            opts.action_backend = Some(tcp_endpoint(
                &self.advertise_host,
                port_u16(tcp.action_backend, "action_backend")?,
            ));
        }
        Ok(())
    }

    fn apply_ipc(&self, opts: &mut NodeOptions) -> Result<()> {
        let dir = self.ipc_dir.as_deref().ok_or_else(|| {
            BusError::Protocol(
                "discovery announce has no ipc_dir (broker may be --tcp-only)".into(),
            )
        })?;
        if opts.message_xsub.is_none() {
            opts.message_xsub = Some(ipc_endpoint_in(dir, XSUB_CHANNEL));
        }
        if opts.message_xpub.is_none() {
            opts.message_xpub = Some(ipc_endpoint_in(dir, XPUB_CHANNEL));
        }
        if opts.service_frontend.is_none() {
            opts.service_frontend = Some(ipc_endpoint_in(dir, SERVICE_FRONTEND_CHANNEL));
        }
        if opts.service_backend.is_none() {
            opts.service_backend = Some(ipc_endpoint_in(dir, SERVICE_BACKEND_CHANNEL));
        }
        if opts.action_frontend.is_none() {
            opts.action_frontend = Some(ipc_endpoint_in(dir, ACTION_FRONTEND_CHANNEL));
        }
        if opts.action_backend.is_none() {
            opts.action_backend = Some(ipc_endpoint_in(dir, ACTION_BACKEND_CHANNEL));
        }
        Ok(())
    }

    fn apply_inproc(&self, opts: &mut NodeOptions) -> Result<()> {
        let prefix = self.inproc_prefix.as_deref().ok_or_else(|| {
            BusError::Protocol(
                "discovery announce has no inproc_prefix (broker may be --tcp-only)".into(),
            )
        })?;
        if opts.message_xsub.is_none() {
            opts.message_xsub = Some(inproc_endpoint_with_prefix(prefix, XSUB_CHANNEL));
        }
        if opts.message_xpub.is_none() {
            opts.message_xpub = Some(inproc_endpoint_with_prefix(prefix, XPUB_CHANNEL));
        }
        if opts.service_frontend.is_none() {
            opts.service_frontend = Some(inproc_endpoint_with_prefix(
                prefix,
                SERVICE_FRONTEND_CHANNEL,
            ));
        }
        if opts.service_backend.is_none() {
            opts.service_backend =
                Some(inproc_endpoint_with_prefix(prefix, SERVICE_BACKEND_CHANNEL));
        }
        if opts.action_frontend.is_none() {
            opts.action_frontend =
                Some(inproc_endpoint_with_prefix(prefix, ACTION_FRONTEND_CHANNEL));
        }
        if opts.action_backend.is_none() {
            opts.action_backend = Some(inproc_endpoint_with_prefix(prefix, ACTION_BACKEND_CHANNEL));
        }
        Ok(())
    }

    fn apply_grpc(&self, opts: &mut NodeOptions) -> Result<()> {
        let url = self
            .ws_url
            .as_deref()
            .ok_or_else(|| BusError::Protocol("discovery announce has no ws_url / apiUrl".into()))?;
        if opts.ws_url.is_none() {
            opts.ws_url = Some(url.to_string());
        }
        Ok(())
    }
}

fn port_u16(port: u32, name: &str) -> Result<u16> {
    u16::try_from(port)
        .map_err(|_| BusError::Protocol(format!("discovery announce invalid {name} port {port}")))
}

impl NodeOptions {
    /// Wait for a broker discover response, then apply it onto this options value.
    pub fn discover(self, opts: DiscoverOpts) -> Result<Self> {
        wait(opts)?.apply(self)
    }
}
