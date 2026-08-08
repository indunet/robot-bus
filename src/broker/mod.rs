//! Broker routing processes (message / service / action bus).

pub mod action_bus;
pub mod federation;
pub mod handle;
pub mod message_bus;
pub mod parse_config;
pub mod service_bus;

pub use crate::discovery::DiscoveryConfig;
pub use action_bus::ActionPeer;
pub use federation::{FederationPeerEndpoints, apply_api_peers, resolve_peer_from_api};
pub use handle::{RobotBusBroker, RobotBusConfig};
pub use message_bus::MessagePeer;
pub use parse_config::{apply_federation_opts, parse_robot_bus_config, robot_bus_broker_help};
pub use service_bus::ServicePeer;

#[cfg(feature = "grpc")]
pub use handle::GrpcBrokerConfig;

#[cfg(feature = "console")]
pub use handle::ConsoleBrokerConfig;
