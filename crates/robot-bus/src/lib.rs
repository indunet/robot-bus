//! ZeroMQ participant SDK: Node, executors, and topic / service / action clients.
//!
//! The broker, WebSocket server, console, and ROS 2 bridge live in separate crates
//! (`robot-bus-broker`, `robot-bus-ros2`). Message types are re-exported from
//! `robot-bus-msgs`.

#[doc(hidden)]
pub mod action_bus;
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
#[doc(hidden)]
pub mod transports;
#[doc(hidden)]
pub mod wire;
#[doc(hidden)]
pub mod worker_thread;
#[doc(hidden)]
pub mod zmq_helpers;

pub use robot_bus_msgs::{
    Action, ActionOutcome, Service, ackermann_msgs, action, action_msgs, apriltag_msgs,
    builtin_interfaces, control_msgs, diagnostic_msgs, example_interfaces, foxglove_msgs,
    geometry_msgs, lifecycle_msgs, map_msgs, nav2_msgs, nav_msgs, robot_bus_interfaces,
    sensor_msgs, shape_msgs, std_msgs, std_srvs, stereo_msgs, tf2_msgs, trajectory_msgs,
    unique_identifier_msgs, vision_msgs, visualization_msgs,
};
pub use robot_bus_msgs::typed;

#[doc(hidden)]
pub use action_bus::{ActionClient, ActionKind, ActionMessage, ActionWorker};
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
#[doc(hidden)]
pub use transports::{
    BindAllOpts, DEFAULT_API_PORT, action_backend_endpoint, action_frontend_endpoint, bind_all,
    bind_tcp, format_endpoints, inproc_endpoint, ipc_endpoint, message_xpub_endpoint,
    message_xsub_endpoint, service_backend_endpoint, service_frontend_endpoint,
};
pub use zmq_helpers::HighWaterMark;
