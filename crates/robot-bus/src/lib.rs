//! ZeroMQ message bus: participant SDK (Node, executors, topic / service / action
//! clients) and the broker, WebSocket server, and embedded console.
//!
//! The SDK is always available. The broker side is behind the `broker` feature and
//! its extensions `ws`, `console`, and `demo-tank`; all are on by default. Programs
//! that only create `Node`s can use `default-features = false`.
//!
//! The ROS 2 bridge lives in the separate `robot-bus-ros2` crate. Message types are
//! re-exported from `robot-bus-msgs`.

#[doc(hidden)]
pub mod action_bus;
#[cfg(feature = "broker")]
pub mod broker;
#[cfg(feature = "console-api")]
pub mod console;
#[doc(hidden)]
pub mod console_topics;
pub mod discovery;
pub mod errors;
#[doc(hidden)]
pub mod message_bus;
pub mod runtime;
#[doc(hidden)]
pub mod service_bus;
#[doc(hidden)]
pub mod shutdown;
#[cfg(feature = "demo-tank")]
pub mod tank;
#[doc(hidden)]
pub mod transports;
#[doc(hidden)]
pub mod wire;
#[doc(hidden)]
pub mod worker_thread;
#[cfg(feature = "ws")]
pub mod ws;
#[doc(hidden)]
pub mod zmq_helpers;

pub use robot_bus_msgs::typed;
pub use robot_bus_msgs::{
    Action, ActionOutcome, Service, ackermann_msgs, action, action_msgs, apriltag_msgs,
    builtin_interfaces, control_msgs, diagnostic_msgs, example_interfaces, foxglove_msgs,
    geometry_msgs, lifecycle_msgs, map_msgs, nav_msgs, nav2_msgs, robot_bus_interfaces,
    sensor_msgs, shape_msgs, std_msgs, std_srvs, stereo_msgs, tf2_msgs, trajectory_msgs,
    unique_identifier_msgs, vision_msgs, visualization_msgs,
};

#[doc(hidden)]
pub use action_bus::{ActionClient, ActionKind, ActionMessage, ActionWorker};
#[cfg(feature = "console-api")]
pub use broker::ConsoleBrokerConfig;
#[cfg(feature = "ws")]
pub use broker::WsConfig;
#[cfg(feature = "broker")]
pub use broker::{
    ActionPeer, DiscoveryConfig, FederationPeerEndpoints, MessagePeer, RobotBusBroker,
    RobotBusConfig, ServicePeer, apply_api_peers, apply_federation_opts, parse_robot_bus_config,
    resolve_peer_from_api, robot_bus_broker_help,
};
pub use discovery::{
    BrokerAnnouncement, DEFAULT_API_DISCOVER_PATH, DEFAULT_WS_RPC_PATH, DiscoverOpts,
    DiscoverResponse, MAGIC as DISCOVERY_MAGIC, SCHEMA_VERSION as DISCOVERY_SCHEMA_VERSION,
    fetch_discover, wait as discover_wait, with_ws_rpc_path,
};
#[allow(deprecated)]
pub use discovery::{DEFAULT_DISCOVERY_PORT, DEFAULT_MULTICAST_ADDR};
pub use errors::{BusError, Result, parse_error_body};
#[doc(hidden)]
pub use message_bus::{Publisher, Subscriber};
pub use runtime::{
    ActionGoalContext, ActionGoalHandler, ActionGoalLiveHandler, CallbackGroup, CallbackGroupType,
    ConnectionState, Context, Executor, ExecutorHandle, GoalHandle, ListParametersResult,
    MessageCallback, MultiThreadedExecutor, Node, NodeActionClient, NodeActionClientRaw,
    NodeActionServer, NodeOptions, NodeService, NodeServiceClient, NodeServiceClientRaw,
    PARAMETER_DEPTH_RECURSIVE, Parameter, ParameterValue, QOS_PROFILE_DEFAULT, QosProfile,
    RawActionFeedbackCallback, RawGoalHandle, ServiceHandler, ShutdownHandle,
    SingleThreadedExecutor, SubscriptionHandle, SubscriptionOverflowPolicy, TimerCallback,
    TimerHandle, TopicPublisher, TopicPublisherRaw,
};
#[doc(hidden)]
pub use service_bus::{ServiceClient, ServiceWorker};
#[cfg(feature = "demo-tank")]
pub use tank::{
    CMD_VEL_TOPIC, MULTI_WAYPOINT_NAV_ACTION, POINT_NAV_ACTION, POSE_TOPIC, RESET_SERVICE,
    WORLD_SIZE,
};
#[doc(hidden)]
pub use transports::{
    BindAllOpts, DEFAULT_API_PORT, action_backend_endpoint, action_frontend_endpoint, bind_all,
    bind_tcp, format_endpoints, inproc_endpoint, ipc_endpoint, message_xpub_endpoint,
    message_xsub_endpoint, service_backend_endpoint, service_frontend_endpoint,
};
pub use zmq_helpers::HighWaterMark;
