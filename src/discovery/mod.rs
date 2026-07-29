//! UDP multicast broker discovery (control plane).
//!
//! Brokers periodically announce [`BrokerAnnouncement`] as a pure protobuf
//! payload. Clients decode + validate (`magic` / `schema_version`), then
//! [`BrokerAnnouncement::apply`] fills location fields on a user-chosen
//! [`crate::NodeOptions`] transport (`tcp` / `ipc` / `inproc` / `grpc`).

mod announce;
mod config;
mod listen;
mod net;
mod packet;

pub use announce::{spawn_announcer, AnnounceHandle, AnnouncerPayload};
pub use announce::resolve_advertise_host;
pub use config::{
    DiscoveryConfig, DiscoverOpts, DEFAULT_DISCOVERY_INTERVAL, DEFAULT_DISCOVERY_PORT,
    DEFAULT_DISCOVERY_TIMEOUT, DEFAULT_MULTICAST_ADDR, MAGIC, SCHEMA_VERSION,
};
pub use listen::wait;
pub use net::tcp_port_from_bind;
pub use packet::{decode_announce, encode_announce, try_parse_datagram, BrokerAnnouncement};

use crate::errors::{BusError, Result};
use crate::runtime::NodeOptions;
use crate::transports::{
    ipc_endpoint_in, inproc_endpoint_with_prefix, tcp_endpoint, ACTION_BACKEND_CHANNEL,
    ACTION_FRONTEND_CHANNEL, SERVICE_BACKEND_CHANNEL, SERVICE_FRONTEND_CHANNEL, XPUB_CHANNEL,
    XSUB_CHANNEL,
};

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
        Ok(opts)
    }

    fn apply_tcp(&self, opts: &mut NodeOptions) -> Result<()> {
        let tcp = self.tcp.as_ref().ok_or_else(|| {
            BusError::Protocol("discovery announce missing tcp ports".into())
        })?;
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
            opts.service_frontend =
                Some(inproc_endpoint_with_prefix(prefix, SERVICE_FRONTEND_CHANNEL));
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
            opts.action_backend =
                Some(inproc_endpoint_with_prefix(prefix, ACTION_BACKEND_CHANNEL));
        }
        Ok(())
    }

    fn apply_grpc(&self, opts: &mut NodeOptions) -> Result<()> {
        let url = self.grpc_url.as_deref().ok_or_else(|| {
            BusError::Protocol("discovery announce has no grpc_url".into())
        })?;
        if opts.grpc_url.is_none() {
            opts.grpc_url = Some(url.to_string());
        }
        Ok(())
    }
}

fn port_u16(port: u32, name: &str) -> Result<u16> {
    u16::try_from(port).map_err(|_| {
        BusError::Protocol(format!("discovery announce invalid {name} port {port}"))
    })
}

impl NodeOptions {
    /// Wait for a broker announce, then apply it onto this options value.
    pub fn discover(self, opts: DiscoverOpts) -> Result<Self> {
        wait(opts)?.apply(self)
    }
}
