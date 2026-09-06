//! Discovery defaults and broker/client options.

use std::time::Duration;

use crate::transports::DEFAULT_API_PORT;

/// Magic string embedded in every valid protobuf [`super::BrokerAnnouncement`] (legacy UDP).
pub const MAGIC: &str = "RBUS";

/// Supported announce schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Deprecated: former UDP multicast group (kept for API stability).
#[deprecated(note = "UDP discovery removed; use HTTP GET /api/v1/discover")]
pub const DEFAULT_MULTICAST_ADDR: std::net::Ipv4Addr = std::net::Ipv4Addr::new(239, 255, 76, 67);

/// Deprecated: former UDP discovery port.
#[deprecated(note = "UDP discovery removed; use HTTP GET /api/v1/discover")]
pub const DEFAULT_DISCOVERY_PORT: u16 = 15550;

/// Path on the API listen port for broker endpoint discovery.
pub const DEFAULT_API_DISCOVER_PATH: &str = "/api/v1/discover";

/// HTTP path for multiplexed WebSocket RPC on the API listen port.
pub const DEFAULT_WS_RPC_PATH: &str = "/ws-rpc";

/// Append [`DEFAULT_WS_RPC_PATH`] to an HTTP or `ws(s)` origin.
///
/// Already-suffixed `/ws-rpc` is left unchanged. A leftover `/ws` suffix
/// (pre-rename clients) is rewritten to `/ws-rpc`.
pub fn with_ws_rpc_path(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    if trimmed.ends_with(DEFAULT_WS_RPC_PATH) {
        trimmed.to_string()
    } else if let Some(prefix) = trimmed.strip_suffix("/ws") {
        format!("{prefix}{DEFAULT_WS_RPC_PATH}")
    } else {
        format!("{trimmed}{DEFAULT_WS_RPC_PATH}")
    }
}

/// Default client wait timeout.
pub const DEFAULT_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

/// Broker-side discovery settings (advertise host for connectable URLs).
#[derive(Clone, Debug)]
pub struct DiscoveryConfig {
    /// Kept for compatibility; UDP announce is no longer started.
    pub enabled: bool,
    /// Optional soft filter / label (also returned in discover JSON).
    pub domain_id: u32,
    /// Override advertise host (otherwise inferred / `127.0.0.1`).
    pub advertise_host: Option<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            domain_id: 0,
            advertise_host: None,
        }
    }
}

/// Client-side discover options (HTTP against the broker API listen port).
#[derive(Clone, Debug)]
pub struct DiscoverOpts {
    /// Broker API base URL, e.g. `http://127.0.0.1:15560` or `127.0.0.1:15560`.
    pub api_url: String,
    /// When set, only accept this broker id.
    pub broker_id: Option<String>,
    pub timeout: Duration,
}

impl Default for DiscoverOpts {
    fn default() -> Self {
        Self {
            api_url: format!("http://127.0.0.1:{DEFAULT_API_PORT}"),
            broker_id: None,
            timeout: DEFAULT_DISCOVERY_TIMEOUT,
        }
    }
}

impl DiscoverOpts {
    /// Discover against `http://{host}:{DEFAULT_API_PORT}`.
    pub fn for_host(host: impl AsRef<str>) -> Self {
        let host = host.as_ref();
        let host = if host == "localhost" {
            "127.0.0.1"
        } else {
            host
        };
        Self {
            api_url: format!("http://{host}:{DEFAULT_API_PORT}"),
            ..Self::default()
        }
    }

    /// Discover against an explicit API base URL.
    pub fn at(api_url: impl Into<String>) -> Self {
        Self {
            api_url: api_url.into(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_ws_rpc_path_appends_and_rewrites_legacy() {
        assert_eq!(
            with_ws_rpc_path("http://127.0.0.1:15560"),
            "http://127.0.0.1:15560/ws-rpc"
        );
        assert_eq!(
            with_ws_rpc_path("http://127.0.0.1:15560/"),
            "http://127.0.0.1:15560/ws-rpc"
        );
        assert_eq!(
            with_ws_rpc_path("ws://127.0.0.1:15560/ws-rpc"),
            "ws://127.0.0.1:15560/ws-rpc"
        );
        assert_eq!(
            with_ws_rpc_path("ws://127.0.0.1:15560/ws"),
            "ws://127.0.0.1:15560/ws-rpc"
        );
    }
}
