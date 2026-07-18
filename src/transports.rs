//! Multi-transport endpoints: TCP (remote), inproc (same process), ipc (local machine).

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use zmq::Socket;

pub const XSUB_PORT: u16 = 15560;
pub const XPUB_PORT: u16 = 15561;
pub const SERVICE_FRONTEND_PORT: u16 = 15662;
pub const SERVICE_BACKEND_PORT: u16 = 15663;
pub const ACTION_FRONTEND_PORT: u16 = 15664;
pub const ACTION_BACKEND_PORT: u16 = 15665;

pub const XSUB_CHANNEL: &str = "message_bus/xsub";
pub const XPUB_CHANNEL: &str = "message_bus/xpub";
pub const SERVICE_FRONTEND_CHANNEL: &str = "service_bus/frontend";
pub const SERVICE_BACKEND_CHANNEL: &str = "service_bus/backend";
pub const ACTION_FRONTEND_CHANNEL: &str = "action_bus/frontend";
pub const ACTION_BACKEND_CHANNEL: &str = "action_bus/backend";

/// Directory for ipc endpoint files (`ipc:///tmp/robot_bus/*.ipc`).
pub const IPC_DIR: &str = "/tmp/robot_bus";

/// Stable in-process endpoint for a logical channel (e.g. `message_bus/xsub`).
pub fn inproc_endpoint(channel: &str) -> String {
    inproc_endpoint_with_prefix("robot_bus", channel)
}

/// In-process endpoint under a custom prefix (`my_app` or `inproc://my_app`).
pub fn inproc_endpoint_with_prefix(prefix: &str, channel: &str) -> String {
    let prefix = prefix.trim().trim_end_matches('/');
    let base = if prefix.starts_with("inproc://") {
        prefix.to_string()
    } else {
        format!("inproc://{prefix}")
    };
    format!("{base}/{channel}")
}

/// Stable local-machine endpoint for a logical channel.
pub fn ipc_endpoint(channel: &str) -> String {
    ipc_endpoint_in(IPC_DIR, channel)
}

/// IPC endpoint under a custom directory (e.g. `/var/run/robot_bus`).
pub fn ipc_endpoint_in(dir: &str, channel: &str) -> String {
    let file = channel.replace('/', "_");
    let dir = dir.trim().trim_end_matches('/');
    format!("ipc://{dir}/{file}.ipc")
}

pub fn tcp_endpoint(host: &str, port: u16) -> String {
    format!("tcp://{host}:{port}")
}

/// Bind `tcp_endpoint` plus matching inproc/ipc endpoints on the same socket.
pub fn bind_all(socket: &Socket, tcp_endpoint: &str, channel: &str) -> Result<Vec<String>> {
    ensure_ipc_dir()?;
    let endpoints = [
        tcp_endpoint.to_string(),
        inproc_endpoint(channel),
        ipc_endpoint(channel),
    ];
    for ep in &endpoints {
        socket
            .bind(ep)
            .with_context(|| format!("bind {ep}"))?;
    }
    Ok(endpoints.to_vec())
}

/// Format bound endpoints for startup logs.
pub fn format_endpoints(endpoints: &[String]) -> String {
    endpoints.join("\n    ")
}

fn pick_endpoint(host: &str, port: u16, channel: &str, transport: &str) -> std::result::Result<String, String> {
    match transport {
        "tcp" => Ok(tcp_endpoint(host, port)),
        "inproc" => Ok(inproc_endpoint(channel)),
        "ipc" => Ok(ipc_endpoint(channel)),
        other => Err(format!("unknown transport: {other:?}")),
    }
}

pub fn message_xsub_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(host, XSUB_PORT, XSUB_CHANNEL, transport)
}

pub fn message_xpub_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(host, XPUB_PORT, XPUB_CHANNEL, transport)
}

pub fn service_frontend_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        SERVICE_FRONTEND_PORT,
        SERVICE_FRONTEND_CHANNEL,
        transport,
    )
}

pub fn service_backend_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        SERVICE_BACKEND_PORT,
        SERVICE_BACKEND_CHANNEL,
        transport,
    )
}

pub fn action_frontend_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        ACTION_FRONTEND_PORT,
        ACTION_FRONTEND_CHANNEL,
        transport,
    )
}

pub fn action_backend_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        ACTION_BACKEND_PORT,
        ACTION_BACKEND_CHANNEL,
        transport,
    )
}

fn ensure_ipc_dir() -> Result<()> {
    fs::create_dir_all(Path::new(IPC_DIR)).context("create ipc directory")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inproc_uses_robot_bus_namespace() {
        assert_eq!(
            inproc_endpoint("message_bus/xsub"),
            "inproc://robot_bus/message_bus/xsub"
        );
    }

    #[test]
    fn inproc_custom_prefix() {
        assert_eq!(
            inproc_endpoint_with_prefix("my_app", "message_bus/xsub"),
            "inproc://my_app/message_bus/xsub"
        );
        assert_eq!(
            inproc_endpoint_with_prefix("inproc://my_app", "message_bus/xsub"),
            "inproc://my_app/message_bus/xsub"
        );
    }

    #[test]
    fn ipc_maps_channel_to_file() {
        assert_eq!(
            ipc_endpoint("service_bus/frontend"),
            "ipc:///tmp/robot_bus/service_bus_frontend.ipc"
        );
    }

    #[test]
    fn ipc_custom_dir() {
        assert_eq!(
            ipc_endpoint_in("/var/run/robot_bus", "message_bus/xsub"),
            "ipc:///var/run/robot_bus/message_bus_xsub.ipc"
        );
    }
}
