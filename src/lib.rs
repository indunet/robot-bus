//! ZeroMQ message bus: broker routing and participant SDK.

pub mod action_bus;
pub mod bot_sim;
pub mod broker;
pub mod console_topics;
pub mod discovery;
pub mod errors;
pub mod message_bus;
pub mod runtime;
pub mod service_bus;
pub mod shutdown;
pub mod transports;
pub mod typed;
pub mod worker_thread;
pub mod zmq_helpers;

// Keep packages under a private module so prost `super::…` paths resolve;
// re-export at crate root as `robot_bus::sensor_msgs::…`.
mod generated;

pub use generated::{
    action, action_msgs, apriltag_msgs, builtin_interfaces, control_msgs, diagnostic_msgs,
    foxglove_msgs, geometry_msgs, nav_msgs, nav2_msgs, robot_bus_interface, sensor_msgs,
    shape_msgs, std_msgs, std_srvs, stereo_msgs, tf2_msgs, trajectory_msgs, unique_identifier_msgs,
    visualization_msgs,
};
pub use typed::{Action, ActionOutcome, Service};

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "ros2")]
pub mod ros2;

#[cfg(feature = "extension-module")]
mod python_api;

pub use action_bus::{ActionClient, ActionKind, ActionMessage, ActionWorker};
pub use bot_sim::{
    BotSimEndpoints, BotSimHandle, BotSimManager, BotSimSession, BotSimStatus, CMD_VEL_TOPIC,
    MULTI_WAYPOINT_NAV_ACTION, POINT_NAV_ACTION, POSE_TOPIC, RESET_SERVICE, WORLD_SIZE,
};
pub use broker::{
    ActionPeer, DiscoveryConfig, MessagePeer, RobotBusBroker, RobotBusConfig, ServicePeer,
    apply_federation_opts, parse_robot_bus_config, robot_bus_broker_help,
};
pub use discovery::{
    BrokerAnnouncement, DEFAULT_DISCOVERY_PORT, DEFAULT_MULTICAST_ADDR, DiscoverOpts,
    MAGIC as DISCOVERY_MAGIC, SCHEMA_VERSION as DISCOVERY_SCHEMA_VERSION, decode_announce,
    encode_announce, wait as discover_wait,
};

#[cfg(feature = "grpc")]
pub use broker::GrpcBrokerConfig;

#[cfg(feature = "console")]
pub use broker::ConsoleBrokerConfig;
pub use errors::{BusError, Result, parse_error_body};
pub use message_bus::{Publisher, Subscriber};
pub use runtime::{
    ActionGoalHandler, CallbackGroup, CallbackGroupType, Context, Executor, ExecutorHandle,
    GoalHandle, MessageCallback, MultiThreadedExecutor, Node, NodeActionClient,
    NodeActionClientRaw, NodeActionServer, NodeOptions, NodeService, NodeServiceClient,
    NodeServiceClientRaw, Parameter, ParameterValue, RawActionFeedbackCallback, RawGoalHandle,
    ServiceHandler, ShutdownHandle, SingleThreadedExecutor, TimerCallback, TimerHandle,
    TopicPublisher, TopicPublisherRaw,
};
pub use service_bus::{ServiceClient, ServiceWorker};
pub use transports::{
    action_backend_endpoint, action_frontend_endpoint, bind_all, format_endpoints, inproc_endpoint,
    ipc_endpoint, message_xpub_endpoint, message_xsub_endpoint, service_backend_endpoint,
    service_frontend_endpoint,
};
pub use zmq_helpers::HighWaterMark;
