//! Unified poll loop runtime (ROS 2–style executor / node).
//!
//! Recommended: [`Context::new`](crate::Context::new) → [`Node::with_context`](crate::Node::with_context)
//! → [`Node::spin`](crate::Node::spin). Convenience: [`Node::new`](crate::Node::new) (private context).
//! Shared / multi-threaded: attach nodes with `add_node` / `create_node`, then `spin` on the executor.

mod callback_group;
mod context;
mod dispatch;
mod executor;
mod executors;
#[cfg(feature = "ws")]
mod ws_runtime;
mod node;
mod parameters;
mod qos;
mod queues;
mod registrations;
mod timers;
mod topic_callbacks;
mod topic_type_register;
mod topology_register;
mod worker_pool;

pub use callback_group::{CallbackGroup, CallbackGroupType};
pub use context::Context;
pub use executor::{Executor, ShutdownHandle};
pub use executors::{ExecutorHandle, MultiThreadedExecutor, SingleThreadedExecutor};
pub use node::{
    GoalHandle, Node, NodeActionClient, NodeActionClientRaw, NodeActionServer, NodeOptions,
    NodeService, NodeServiceClient, NodeServiceClientRaw, RawActionFeedbackCallback, RawGoalHandle,
    TopicPublisher, TopicPublisherRaw,
};
pub use parameters::{ListParametersResult, Parameter, ParameterValue, PARAMETER_DEPTH_RECURSIVE};
pub use qos::{QOS_PROFILE_DEFAULT, QosProfile};
pub use queues::ActionMessageCallback;
pub use registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
pub use timers::{TimerCallback, TimerHandle};
