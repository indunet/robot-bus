//! Resolve federation peers from a remote broker API listen address.

use anyhow::{Context, Result};

use crate::discovery::{DiscoverResponse, fetch_discover, normalize_api_base};

use super::action_bus::ActionPeer;
use super::message_bus::MessagePeer;
use super::service_bus::ServicePeer;

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
}

/// Resolve federation peers from a remote broker API (`host:port` or URL).
pub fn resolve_peer_from_api(api: &str) -> Result<FederationPeerEndpoints> {
    let base = normalize_api_base(api);
    let disc = fetch_discover(&base, std::time::Duration::from_secs(3))
        .with_context(|| format!("resolve peer from API {base}"))?;
    Ok(FederationPeerEndpoints::from_discover(&disc))
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
