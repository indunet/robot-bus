//! Broker, WebSocket server, embedded console, and the `rbus` CLI.
//!
//! Pair this crate with `robot-bus` when a process hosts the bus. Client-only
//! programs depend on `robot-bus` alone.

pub mod broker;
#[cfg(feature = "console-api")]
pub mod console;
#[cfg(feature = "demo-tank")]
pub mod tank;

#[cfg(feature = "demo-tank")]
pub use tank::{
    CMD_VEL_TOPIC, MULTI_WAYPOINT_NAV_ACTION, POINT_NAV_ACTION, POSE_TOPIC, RESET_SERVICE,
    WORLD_SIZE,
};
#[cfg(feature = "ws")]
pub mod ws;

pub use broker::{
    ActionPeer, DiscoveryConfig, FederationPeerEndpoints, MessagePeer, RobotBusBroker,
    RobotBusConfig, ServicePeer, apply_api_peers, apply_federation_opts, parse_robot_bus_config,
    resolve_peer_from_api, robot_bus_broker_help,
};
#[cfg(feature = "console-api")]
pub use broker::ConsoleBrokerConfig;
#[cfg(feature = "ws")]
pub use broker::WsConfig;
