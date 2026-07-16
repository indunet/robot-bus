//! Unified poll loop runtime (ROS 2–style executor / node).
//!
//! Create a [`SingleThreadedExecutor`] or [`MultiThreadedExecutor`], attach
//! nodes with `create_node`, register callbacks on the [`Node`], then drive
//! them with `spin` / `spin_once` / `spin_some` on the executor.

mod executor;
mod executors;
mod callback_group;
mod dispatch;
mod node;
mod queues;
mod registrations;
mod timers;
mod worker_pool;

pub use callback_group::{CallbackGroup, CallbackGroupType};
pub use executor::{Executor, ShutdownHandle};
pub use executors::{ExecutorHandle, MultiThreadedExecutor, SingleThreadedExecutor};
pub use node::{Node, NodeOptions, TopicPublisher};
pub use queues::ActionMessageCallback;
pub use registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
pub use timers::{TimerCallback, TimerHandle};
