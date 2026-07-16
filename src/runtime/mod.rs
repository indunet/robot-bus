//! Unified poll loop runtime (ROS 2–style executor / node).
//!
//! Register callbacks on [`Executor`] (or the [`Node`] facade), then drive
//! them with `spin` / `spin_once` / `spin_some`.

mod executor;
mod dispatch;
mod node;
mod queues;
mod registrations;
mod timers;
mod worker_pool;

pub use executor::{Executor, ShutdownHandle};
pub use node::{Node, NodeOptions};
pub use queues::ActionMessageCallback;
pub use registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
pub use timers::{TimerCallback, TimerHandle};
