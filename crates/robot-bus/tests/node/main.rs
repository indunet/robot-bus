//! Node, executor, and discovery integration tests.

#[path = "../common/mod.rs"]
mod common;

mod callback_groups;
mod destroy_handles;
mod discovery;
mod executor_wakeup;
mod inproc_context;
mod node_service_action;
mod node_session;
mod runtime_spin;
mod topic_qos;
#[cfg(feature = "console-api")]
mod wait_helpers;
