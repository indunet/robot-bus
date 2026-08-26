//! Multi-transport endpoints: TCP (remote), inproc (same process), ipc (local machine).

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use zmq::Socket;

/// Legacy fixed TCP ports (pre-ephemeral). Prefer discover / explicit binds.
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

/// Default API listen port (gRPC + WS + console + discover).
pub const DEFAULT_API_PORT: u16 = 15570;

/// Options for multi-transport bind (tcp + optional inproc/ipc).
#[derive(Clone, Debug)]
pub struct BindAllOpts {
    /// IPC directory (default [`IPC_DIR`]). Uniquify with broker id for multi-broker hosts.
    pub ipc_dir: String,
    /// Inproc prefix (default `robot_bus`).
    pub inproc_prefix: String,
}

impl Default for BindAllOpts {
    fn default() -> Self {
        Self {
            ipc_dir: IPC_DIR.to_string(),
            inproc_prefix: "robot_bus".to_string(),
        }
    }
}

impl BindAllOpts {
    pub fn for_broker(broker_id: &str) -> Self {
        Self {
            ipc_dir: format!("{IPC_DIR}/{broker_id}"),
            inproc_prefix: "robot_bus".to_string(),
        }
    }
}

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

/// Read the last bound endpoint (resolves `tcp://…:0` to the OS-assigned port).
pub fn last_endpoint(socket: &Socket) -> Result<String> {
    match socket
        .get_last_endpoint()
        .context("get_last_endpoint after bind")?
    {
        Ok(s) => Ok(s),
        Err(_) => bail!("last endpoint is not valid UTF-8"),
    }
}

/// Bind TCP (resolving `:0`), then matching inproc/ipc aliases on the same socket.
///
/// Returns `(resolved_tcp, all_endpoints)` where `resolved_tcp` is the concrete
/// `tcp://…:port` after ephemeral allocation.
pub fn bind_all(
    socket: &Socket,
    tcp_bind: &str,
    channel: &str,
    opts: &BindAllOpts,
) -> Result<(String, Vec<String>)> {
    ensure_ipc_dir(&opts.ipc_dir)?;
    socket
        .bind(tcp_bind)
        .with_context(|| format!("bind {tcp_bind}"))?;
    let resolved_tcp = last_endpoint(socket)?;
    let inproc = inproc_endpoint_with_prefix(&opts.inproc_prefix, channel);
    let ipc = ipc_endpoint_in(&opts.ipc_dir, channel);
    socket
        .bind(&inproc)
        .with_context(|| format!("bind {inproc}"))?;
    socket.bind(&ipc).with_context(|| format!("bind {ipc}"))?;
    Ok((resolved_tcp.clone(), vec![resolved_tcp, inproc, ipc]))
}

/// Bind a single TCP endpoint and return the resolved address (`:0` → real port).
pub fn bind_tcp(socket: &Socket, tcp_bind: &str) -> Result<String> {
    socket
        .bind(tcp_bind)
        .with_context(|| format!("bind {tcp_bind}"))?;
    last_endpoint(socket)
}

/// Format bound endpoints for startup logs.
pub fn format_endpoints(endpoints: &[String]) -> String {
    endpoints.join("\n    ")
}

fn pick_endpoint(
    host: &str,
    port: u16,
    channel: &str,
    transport: &str,
) -> std::result::Result<String, String> {
    match transport {
        "tcp" => {
            if port == 0 {
                return Err(
                    "tcp endpoint port is 0; discover via the broker API (default http://127.0.0.1:15570/api/v1/discover) or set endpoints explicitly"
                        .into(),
                );
            }
            Ok(tcp_endpoint(host, port))
        }
        "inproc" => Ok(inproc_endpoint(channel)),
        "ipc" => Ok(ipc_endpoint(channel)),
        other => Err(format!("unknown transport: {other:?}")),
    }
}

/// Legacy helper: fixed historical TCP ports. Prefer HTTP discover for ephemeral brokers.
pub fn message_xsub_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(host, XSUB_PORT, XSUB_CHANNEL, transport)
}

pub fn message_xpub_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(host, XPUB_PORT, XPUB_CHANNEL, transport)
}

pub fn service_frontend_endpoint(
    host: &str,
    transport: &str,
) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        SERVICE_FRONTEND_PORT,
        SERVICE_FRONTEND_CHANNEL,
        transport,
    )
}

pub fn service_backend_endpoint(
    host: &str,
    transport: &str,
) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        SERVICE_BACKEND_PORT,
        SERVICE_BACKEND_CHANNEL,
        transport,
    )
}

pub fn action_frontend_endpoint(
    host: &str,
    transport: &str,
) -> std::result::Result<String, String> {
    pick_endpoint(
        host,
        ACTION_FRONTEND_PORT,
        ACTION_FRONTEND_CHANNEL,
        transport,
    )
}

pub fn action_backend_endpoint(host: &str, transport: &str) -> std::result::Result<String, String> {
    pick_endpoint(host, ACTION_BACKEND_PORT, ACTION_BACKEND_CHANNEL, transport)
}

fn ensure_ipc_dir(dir: &str) -> Result<()> {
    fs::create_dir_all(Path::new(dir)).with_context(|| format!("create ipc directory {dir}"))?;
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

    #[test]
    fn broker_ipc_dir_is_namespaced() {
        let opts = BindAllOpts::for_broker("abc");
        assert_eq!(opts.ipc_dir, "/tmp/robot_bus/abc");
        assert_eq!(opts.inproc_prefix, "robot_bus");
    }
}
