//! Resolve federation peers from a remote broker API listen address.

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::discovery::{DiscoverResponse, fetch_discover, normalize_api_base};
use crate::transports::{
    ACTION_BACKEND_CHANNEL, SERVICE_BACKEND_CHANNEL, XPUB_CHANNEL, XSUB_CHANNEL, ipc_endpoint_in,
};

use super::action_bus::ActionPeer;
use super::message_bus::MessagePeer;
use super::service_bus::ServicePeer;

/// `--peer` prefix selecting the peer's IPC endpoints (same host only).
pub const IPC_PEER_PREFIX: &str = "ipc+";

/// ZMQ peer endpoints resolved from a remote broker's `GET /api/v1/discover`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FederationPeerEndpoints {
    pub broker_id: String,
    pub api_url: String,
    pub message: MessagePeer,
    pub service: ServicePeer,
    pub action: ActionPeer,
}

impl FederationPeerEndpoints {
    pub fn from_discover(disc: &DiscoverResponse) -> Self {
        Self {
            broker_id: disc.broker_id.clone(),
            api_url: disc.api_url.clone(),
            message: MessagePeer {
                xpub: disc.message_xpub.clone(),
                xsub: disc.message_xsub.clone(),
            },
            service: ServicePeer {
                backend: disc.service_backend.clone(),
                broker_id: disc.broker_id.clone(),
            },
            action: ActionPeer {
                backend: disc.action_backend.clone(),
                broker_id: disc.broker_id.clone(),
            },
        }
    }

    /// Like [`Self::from_discover`], but connect over the peer's `ipc://` endpoints.
    ///
    /// Fails when the peer advertises no `ipc_dir` (e.g. `--tcp-only`) or its socket
    /// files are not visible here (different host, container, or filesystem namespace).
    pub fn from_discover_ipc(disc: &DiscoverResponse) -> Result<Self> {
        let dir = disc
            .ipc_dir
            .as_deref()
            .filter(|d| !d.is_empty())
            .with_context(|| {
                format!(
                    "peer {} advertises no IPC endpoints (started with --tcp-only?)",
                    disc.broker_id
                )
            })?;
        let endpoint = |channel: &str| -> Result<String> {
            let file = Path::new(dir).join(format!("{}.ipc", channel.replace('/', "_")));
            if !file.exists() {
                bail!(
                    "IPC socket {} of peer {} not found on this host; IPC federation needs both \
                     brokers on the same machine and filesystem (drop the {IPC_PEER_PREFIX} prefix to use TCP)",
                    file.display(),
                    disc.broker_id
                );
            }
            Ok(ipc_endpoint_in(dir, channel))
        };
        Ok(Self {
            broker_id: disc.broker_id.clone(),
            api_url: disc.api_url.clone(),
            message: MessagePeer {
                xpub: endpoint(XPUB_CHANNEL)?,
                xsub: endpoint(XSUB_CHANNEL)?,
            },
            service: ServicePeer {
                backend: endpoint(SERVICE_BACKEND_CHANNEL)?,
                broker_id: disc.broker_id.clone(),
            },
            action: ActionPeer {
                backend: endpoint(ACTION_BACKEND_CHANNEL)?,
                broker_id: disc.broker_id.clone(),
            },
        })
    }
}

/// Resolve federation peers from a remote broker API (`host:port` or URL).
///
/// With the [`IPC_PEER_PREFIX`] (`ipc+host:port`), the API is still reached over
/// HTTP but the ZMQ links use the peer's IPC endpoints.
pub fn resolve_peer_from_api(api: &str) -> Result<FederationPeerEndpoints> {
    let (use_ipc, api) = match api.strip_prefix(IPC_PEER_PREFIX) {
        Some(rest) => (true, rest),
        None => (false, api),
    };
    let base = normalize_api_base(api);
    let disc = fetch_discover(&base, std::time::Duration::from_secs(3))
        .with_context(|| format!("resolve peer from API {base}"))?;
    if use_ipc {
        FederationPeerEndpoints::from_discover_ipc(&disc)
    } else {
        Ok(FederationPeerEndpoints::from_discover(&disc))
    }
}

/// Apply `--peer` API URLs onto a [`super::RobotBusConfig`] (message/service/action peers).
pub fn apply_api_peers(config: &mut super::RobotBusConfig, peers: &[String]) -> Result<()> {
    for peer in peers {
        let resolved =
            resolve_peer_from_api(peer).with_context(|| format!("invalid --peer {peer}"))?;
        config.message.peers.push(resolved.message);
        config.service.peers.push(resolved.service);
        config.action.peers.push(resolved.action);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disc(ipc_dir: Option<String>) -> DiscoverResponse {
        DiscoverResponse {
            broker_id: "broker-b".into(),
            domain_id: 0,
            advertise_host: "127.0.0.1".into(),
            api_url: "http://127.0.0.1:15560".into(),
            message_xsub: "tcp://127.0.0.1:1".into(),
            message_xpub: "tcp://127.0.0.1:2".into(),
            service_frontend: "tcp://127.0.0.1:3".into(),
            service_backend: "tcp://127.0.0.1:4".into(),
            action_frontend: "tcp://127.0.0.1:5".into(),
            action_backend: "tcp://127.0.0.1:6".into(),
            ipc_dir,
            inproc_prefix: None,
            console_url: None,
        }
    }

    #[test]
    fn ipc_requires_advertised_dir() {
        let err = FederationPeerEndpoints::from_discover_ipc(&disc(None)).unwrap_err();
        assert!(err.to_string().contains("--tcp-only"), "{err}");
    }

    #[test]
    fn ipc_requires_local_socket_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_owned();
        let err = FederationPeerEndpoints::from_discover_ipc(&disc(Some(path))).unwrap_err();
        assert!(err.to_string().contains("not found on this host"), "{err}");
    }

    #[test]
    fn ipc_uses_peer_dir_endpoints() {
        let dir = tempfile::tempdir().unwrap();
        for channel in [
            XPUB_CHANNEL,
            XSUB_CHANNEL,
            SERVICE_BACKEND_CHANNEL,
            ACTION_BACKEND_CHANNEL,
        ] {
            std::fs::write(
                dir.path()
                    .join(format!("{}.ipc", channel.replace('/', "_"))),
                b"",
            )
            .unwrap();
        }
        let path = dir.path().to_str().unwrap().to_owned();
        let p = FederationPeerEndpoints::from_discover_ipc(&disc(Some(path.clone()))).unwrap();
        assert_eq!(p.message.xpub, ipc_endpoint_in(&path, XPUB_CHANNEL));
        assert_eq!(p.message.xsub, ipc_endpoint_in(&path, XSUB_CHANNEL));
        assert_eq!(
            p.service.backend,
            ipc_endpoint_in(&path, SERVICE_BACKEND_CHANNEL)
        );
        assert_eq!(
            p.action.backend,
            ipc_endpoint_in(&path, ACTION_BACKEND_CHANNEL)
        );
        assert_eq!(p.service.broker_id, "broker-b");
    }
}
