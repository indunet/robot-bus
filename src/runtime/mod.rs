//! Unified poll loop runtime (ROS 2–style executor / node).
//!
//! Register callbacks on [`BusRuntime`] (or the [`Node`] facade), then drive
//! them with `spin` / `spin_once` / `spin_some`.

mod bus_runtime;
mod dispatch;
mod node;
mod queues;
mod registrations;
mod timers;
mod worker_pool;

pub use bus_runtime::{BusRuntime, ShutdownHandle};
pub use node::Node;
pub use queues::ActionMessageCallback;
pub use registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
pub use timers::{TimerCallback, TimerHandle};
