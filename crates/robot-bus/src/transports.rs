//! Multi-transport endpoints: TCP (remote), inproc (same process), ipc (local machine).

use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use zmq::Socket;

/// Legacy fixed TCP ports for client helpers when not using HTTP discover.
/// Distinct from [`DEFAULT_API_PORT`] (console / WS / discover). Prefer discover / explicit binds.
pub const XSUB_PORT: u16 = 15580;
pub const XPUB_PORT: u16 = 15581;
pub const SERVICE_FRONTEND_PORT: u16 = 15662;
pub const SERVICE_BACKEND_PORT: u16 = 15663;
pub const ACTION_FRONTEND_PORT: u16 = 15664;
pub const ACTION_BACKEND_PORT: u16 = 15665;

pub const XSUB_CHANNEL: &str = "message_bus/xsub";
pub const XPUB_CHANNEL: &str = "message_bus/xpub";
/// Federation channel of the message bus (ROUTER joined brokers connect to).
pub const MESSAGE_FED_CHANNEL: &str = "message_bus/fed";
pub const SERVICE_FRONTEND_CHANNEL: &str = "service_bus/frontend";
pub const SERVICE_BACKEND_CHANNEL: &str = "service_bus/backend";
pub const ACTION_FRONTEND_CHANNEL: &str = "action_bus/frontend";
pub const ACTION_BACKEND_CHANNEL: &str = "action_bus/backend";

/// Historical IPC directory on hosts with `/tmp`. Prefer [`default_ipc_dir`].
pub const IPC_DIR: &str = "/tmp/robot_bus";

/// Default directory for standalone buses and legacy fixed IPC endpoints.
pub fn default_ipc_dir() -> PathBuf {
    std::env::temp_dir().join("robot_bus")
}

/// Default API listen port (gRPC + WS + console + discover).
pub const DEFAULT_API_PORT: u16 = 15560;

/// Options for multi-transport bind (tcp + optional inproc/ipc).
#[derive(Clone, Debug)]
pub struct BindAllOpts {
    /// IPC directory (default [`default_ipc_dir`]).
    pub ipc_dir: String,
    /// Inproc prefix (default `robot_bus`).
    pub inproc_prefix: String,
}

impl Default for BindAllOpts {
    fn default() -> Self {
        Self {
            ipc_dir: default_ipc_dir().to_string_lossy().into_owned(),
            inproc_prefix: "robot_bus".to_string(),
        }
    }
}

impl BindAllOpts {
    pub fn for_broker(broker_id: &str) -> Self {
        Self {
            ipc_dir: default_ipc_dir()
                .join(broker_id)
                .to_string_lossy()
                .into_owned(),
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
    ipc_endpoint_in(&default_ipc_dir().to_string_lossy(), channel)
}

/// IPC endpoint under a custom directory (e.g. `/var/run/robot_bus`).
pub fn ipc_endpoint_in(dir: &str, channel: &str) -> String {
    let file = channel.replace('/', "_");
    let path = Path::new(dir).join(format!("{file}.ipc"));
    format!("ipc://{}", path.display())
}

/// Swap the channel file of an `ipc://` endpoint built by [`ipc_endpoint_in`]
/// (e.g. `…/message_bus_xpub.ipc` → `…/message_bus_xsub.ipc`).
///
/// Returns `None` when `endpoint` is not an IPC endpoint for `from_channel`.
pub fn ipc_sibling_endpoint(
    endpoint: &str,
    from_channel: &str,
    to_channel: &str,
) -> Option<String> {
    if !endpoint.starts_with("ipc://") {
        return None;
    }
    let from_file = format!("{}.ipc", from_channel.replace('/', "_"));
    let base = endpoint.strip_suffix(&from_file)?;
    if !(base.ends_with('/') || base.ends_with('\\')) {
        return None;
    }
    Some(format!("{base}{}.ipc", to_channel.replace('/', "_")))
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
    socket
        .bind(tcp_bind)
        .with_context(|| format!("bind {tcp_bind}"))?;
    let resolved_tcp = last_endpoint(socket)?;
    let inproc = inproc_endpoint_with_prefix(&opts.inproc_prefix, channel);
    socket
        .bind(&inproc)
        .with_context(|| format!("bind {inproc}"))?;
    let mut endpoints = vec![resolved_tcp.clone(), inproc];
    if ipc_supported() {
        ensure_ipc_dir(&opts.ipc_dir)?;
        let ipc = ipc_endpoint_in(&opts.ipc_dir, channel);
        socket.bind(&ipc).with_context(|| format!("bind {ipc}"))?;
        endpoints.push(ipc);
    }
    Ok((resolved_tcp, endpoints))
}

/// Whether libzmq and the host runtime support the IPC transport.
pub fn ipc_supported() -> bool {
    if zmq::has("ipc") != Some(true) {
        return false;
    }
    #[cfg(windows)]
    {
        static SUPPORTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *SUPPORTED.get_or_init(|| {
            socket2::Socket::new_raw(socket2::Domain::UNIX, socket2::Type::STREAM, None).is_ok()
        })
    }
    #[cfg(not(windows))]
    true
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
                    "tcp endpoint port is 0; discover via the broker API (default http://127.0.0.1:15560/api/v1/discover) or set endpoints explicitly"
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
            format!(
                "ipc://{}",
                default_ipc_dir().join("service_bus_frontend.ipc").display()
            )
        );
    }

    #[test]
    fn ipc_custom_dir() {
        assert_eq!(
            ipc_endpoint_in("/var/run/robot_bus", "message_bus/xsub"),
            format!(
                "ipc://{}",
                Path::new("/var/run/robot_bus")
                    .join("message_bus_xsub.ipc")
                    .display()
            )
        );
    }

    #[test]
    fn ipc_sibling_swaps_channel_file() {
        let xpub = ipc_endpoint_in("/tmp/rb-x", XPUB_CHANNEL);
        assert_eq!(
            ipc_sibling_endpoint(&xpub, XPUB_CHANNEL, XSUB_CHANNEL),
            Some(ipc_endpoint_in("/tmp/rb-x", XSUB_CHANNEL))
        );
        assert_eq!(
            ipc_sibling_endpoint(&xpub, XSUB_CHANNEL, XPUB_CHANNEL),
            None
        );
        assert_eq!(
            ipc_sibling_endpoint(
                "ipc:///tmp/rb-x/other_message_bus_xpub.ipc",
                XPUB_CHANNEL,
                XSUB_CHANNEL
            ),
            None
        );
        assert_eq!(
            ipc_sibling_endpoint("tcp://127.0.0.1:1", XPUB_CHANNEL, XSUB_CHANNEL),
            None
        );
    }

    #[test]
    fn broker_ipc_dir_is_namespaced() {
        let opts = BindAllOpts::for_broker("abc");
        assert_eq!(Path::new(&opts.ipc_dir), default_ipc_dir().join("abc"));
        assert_eq!(opts.inproc_prefix, "robot_bus");
    }
}
