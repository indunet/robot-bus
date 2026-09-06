//! ZeroMQ message bus: broker routing and participant SDK.

pub mod action_bus;
pub mod broker;
pub mod console_topics;
pub mod discovery;
pub mod errors;
pub mod lazy_subscribe;
pub mod message_bus;
pub mod runtime;
pub mod service_bus;
pub mod shutdown;
pub mod tank;
pub mod transports;
pub mod typed;
pub mod worker_thread;
pub mod zmq_helpers;

// Keep packages under a private module so prost `super::…` paths resolve;
// re-export at crate root as `robot_bus::sensor_msgs::…`.
mod generated;

pub use generated::{
    ackermann_msgs, action, action_msgs, apriltag_msgs, builtin_interfaces, control_msgs,
    diagnostic_msgs, example_interfaces, foxglove_msgs, geometry_msgs, lifecycle_msgs, map_msgs,
    nav_msgs, nav2_msgs, robot_bus_interfaces, sensor_msgs, shape_msgs, std_msgs, std_srvs,
    stereo_msgs, tf2_msgs, trajectory_msgs, unique_identifier_msgs, vision_msgs,
    visualization_msgs,
};
pub use typed::{Action, ActionOutcome, Service};

#[cfg(feature = "ws")]
pub mod ws_gateway;

#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "ros2")]
pub mod ros2_bridge;

#[cfg(feature = "extension-module")]
mod python_api;

pub use action_bus::{ActionClient, ActionKind, ActionMessage, ActionWorker};
pub use broker::{
    ActionPeer, DiscoveryConfig, FederationPeerEndpoints, MessagePeer, RobotBusBroker,
    RobotBusConfig, ServicePeer, apply_api_peers, apply_federation_opts, parse_robot_bus_config,
    resolve_peer_from_api, robot_bus_broker_help,
};
pub use discovery::{
    BrokerAnnouncement, DEFAULT_API_DISCOVER_PATH, DEFAULT_WS_RPC_PATH, DiscoverOpts,
    DiscoverResponse, MAGIC as DISCOVERY_MAGIC, SCHEMA_VERSION as DISCOVERY_SCHEMA_VERSION,
    decode_announce, encode_announce, fetch_discover, wait as discover_wait, with_ws_rpc_path,
};
pub use tank::{
    CMD_VEL_TOPIC, MULTI_WAYPOINT_NAV_ACTION, POINT_NAV_ACTION, POSE_TOPIC, RESET_SERVICE,
    TankEndpoints, TankHandle, TankManager, TankSession, TankStatus, WORLD_SIZE,
};

#[allow(deprecated)]
pub use discovery::{DEFAULT_DISCOVERY_PORT, DEFAULT_MULTICAST_ADDR};

#[cfg(feature = "ws")]
pub use broker::WsGatewayConfig;

#[cfg(feature = "console")]
pub use broker::ConsoleBrokerConfig;
pub use errors::{BusError, Result, parse_error_body};
pub use lazy_subscribe::{CONSOLE_DETECT_TIMEOUT, should_enable_ros_subscription};
pub use message_bus::{Publisher, Subscriber};
pub use runtime::{
    ActionGoalContext, ActionGoalHandler, ActionGoalLiveHandler, CallbackGroup, CallbackGroupType,
    ConnectionState, Context, Executor, ExecutorHandle, GoalHandle, ListParametersResult,
    MessageCallback, MultiThreadedExecutor, Node, NodeActionClient, NodeActionClientRaw,
    NodeActionServer, NodeOptions, NodeService, NodeServiceClient, NodeServiceClientRaw,
    PARAMETER_DEPTH_RECURSIVE, Parameter, ParameterValue, QOS_PROFILE_DEFAULT, QosProfile,
    RawActionFeedbackCallback, RawGoalHandle, ServiceHandler, ShutdownHandle,
    SingleThreadedExecutor, SubscriptionHandle, TimerCallback, TimerHandle, TopicPublisher,
    TopicPublisherRaw,
};
pub use service_bus::{ServiceClient, ServiceWorker};
pub use transports::{
    BindAllOpts, DEFAULT_API_PORT, action_backend_endpoint, action_frontend_endpoint, bind_all,
    bind_tcp, format_endpoints, inproc_endpoint, ipc_endpoint, message_xpub_endpoint,
    message_xsub_endpoint, service_backend_endpoint, service_frontend_endpoint,
};
pub use zmq_helpers::HighWaterMark;
