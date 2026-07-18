//! Unified poll loop runtime (ROS 2–style executor / node).
//!
//! Simple path: [`Node::new`] → register callbacks → [`Node::spin`] (owns a
//! [`SingleThreadedExecutor`] lazily). Shared / multi-threaded: attach nodes
//! with `add_node` / `create_node`, then `spin` on the executor.

mod executor;
mod executors;
mod callback_group;
mod dispatch;
#[cfg(feature = "grpc")]
mod grpc_runtime;
mod node;
mod queues;
mod registrations;
mod timers;
mod worker_pool;

pub use callback_group::{CallbackGroup, CallbackGroupType};
pub use executor::{Executor, ShutdownHandle};
pub use executors::{ExecutorHandle, MultiThreadedExecutor, SingleThreadedExecutor};
pub use node::{
    Node, NodeActionClient, NodeActionClientRaw, NodeActionServer, NodeOptions, NodeService,
    NodeServiceClient, NodeServiceClientRaw, TopicPublisher, TopicPublisherRaw,
};
pub use queues::ActionMessageCallback;
pub use registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
pub use timers::{TimerCallback, TimerHandle};
