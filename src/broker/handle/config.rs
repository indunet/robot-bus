//! Config types for [`super::RobotBusBroker`].

use std::net::SocketAddr;

use super::super::action_bus::ActionBusConfig;
use super::super::message_bus::BusConfig;
use super::super::service_bus::ServiceBusConfig;
use crate::discovery::DiscoveryConfig;

/// WebSocket RPC listen options (feature `ws`, enabled by default).
#[cfg(feature = "ws")]
#[derive(Clone, Debug)]
pub struct WsGatewayConfig {
    pub listen: SocketAddr,
    /// When empty, allow any origin (local-dev default).
    pub cors_origins: Vec<String>,
}

#[cfg(feature = "ws")]
impl Default for WsGatewayConfig {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:15570".parse().expect("default API listen"),
            cors_origins: Vec::new(),
        }
    }
}

/// Embedded Web console HTTP options (feature `console`, enabled by default).
///
/// When the `ws` feature is also enabled, the console UI + REST API are served
/// on [`WsGatewayConfig::listen`] instead — WebSocket RPC (`/ws`) and the console
/// share one port. `listen` here only takes effect when `ws` is disabled (or
/// this crate is built console-only).
#[cfg(feature = "console")]
#[derive(Clone, Debug)]
pub struct ConsoleBrokerConfig {
    /// When false, the console is not started.
    pub enabled: bool,
    /// When false, the in-console tank demo is hidden and cannot be started
    /// (`--no-tank`). Default true for local / sim use.
    pub tank_enabled: bool,
    /// When false, the console docs sidebar entry is hidden (`--no-docs`).
    /// Default true.
    pub docs_enabled: bool,
    /// Listen address used only when the `grpc` feature is disabled.
    pub listen: SocketAddr,
    /// Explicit CORS allowlist for cross-origin browser clients.
    /// Empty (default) disables CORS headers. Never uses `*`.
    pub cors_origins: Vec<String>,
}

#[cfg(feature = "console")]
impl Default for ConsoleBrokerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tank_enabled: true,
            docs_enabled: true,
            listen: "0.0.0.0:15570".parse().expect("default console listen"),
            cors_origins: Vec::new(),
        }
    }
}

/// Configuration for starting all buses (and gRPC / console) in one process.
#[derive(Clone, Debug, Default)]
pub struct RobotBusConfig {
    pub message: BusConfig,
    pub service: ServiceBusConfig,
    pub action: ActionBusConfig,
    pub discovery: DiscoveryConfig,
    #[cfg(feature = "ws")]
    pub ws: WsGatewayConfig,
    #[cfg(feature = "console")]
    pub console: ConsoleBrokerConfig,
}
