//! Unified poll loop runtime (ROS 2–style executor).
//!
//! Register callbacks on [`BusRuntime`], then drive them with
//! [`BusRuntime::spin`] / [`BusRuntime::spin_once`] / [`BusRuntime::spin_some`].

mod bus_runtime;
mod dispatch;
mod queues;
mod registrations;
mod timers;
mod worker_pool;

pub use bus_runtime::{BusRuntime, ShutdownHandle};
pub use queues::ActionMessageCallback;
pub use registrations::{ActionGoalHandler, MessageCallback, ServiceHandler};
pub use timers::{TimerCallback, TimerHandle};
