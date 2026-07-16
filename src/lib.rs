//! ZeroMQ message bus: broker routing and participant SDK.

pub mod action_bus;
pub mod broker;
pub mod errors;
pub mod message_bus;
pub mod runtime;
pub mod service_bus;
pub mod shutdown;
pub mod transports;
pub mod worker_thread;
pub mod zmq_helpers;

// Keep packages under a private module so prost `super::…` paths resolve;
// re-export at crate root as `robot_bus::sensor_msgs::…`.
mod msgs;

pub use msgs::{
    builtin_interfaces, control_msgs, diagnostic_msgs, geometry_msgs, nav2_msgs, nav_msgs,
    sensor_msgs, shape_msgs, std_msgs, std_srvs, tf2_msgs, trajectory_msgs, unique_identifier_msgs,
    visualization_msgs,
};

#[cfg(feature = "grpc")]
pub mod grpc;

#[cfg(feature = "extension-module")]
mod python_api;

pub use action_bus::{ActionClient, ActionKind, ActionMessage, ActionWorker};
pub use broker::{
    ActionBusBroker, MessageBusBroker, RobotBusBroker, RobotBusConfig, ServiceBusBroker,
};
pub use errors::{parse_error_body, BusError, Result};
pub use message_bus::{Publisher, Subscriber};
pub use runtime::{
    Executor, MessageCallback, Node, NodeOptions, ServiceHandler, ShutdownHandle, TimerCallback,
    TimerHandle,
};
pub use service_bus::{ServiceClient, ServiceWorker};
pub use transports::{
    action_backend_endpoint, action_frontend_endpoint, bind_all, format_endpoints, inproc_endpoint,
    ipc_endpoint, message_xpub_endpoint, message_xsub_endpoint, service_backend_endpoint,
    service_frontend_endpoint,
};
pub use zmq_helpers::HighWaterMark;
