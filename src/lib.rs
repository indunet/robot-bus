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

pub use action_bus::{ActionClient, ActionKind, ActionMessage, ActionWorker};
pub use errors::{parse_error_body, BusError, Result};
pub use message_bus::{Publisher, Subscriber};
pub use runtime::BusRuntime;
pub use service_bus::{ServiceClient, ServiceWorker};
pub use transports::{
    action_backend_endpoint, action_frontend_endpoint, bind_all, format_endpoints, inproc_endpoint,
    ipc_endpoint, message_xpub_endpoint, message_xsub_endpoint, service_backend_endpoint,
    service_frontend_endpoint,
};
