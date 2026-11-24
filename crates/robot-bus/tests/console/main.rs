//! Console API, CLI, and tank-session integration tests.

#[path = "../common/mod.rs"]
mod common;

#[cfg(all(feature = "console-api", feature = "ws"))]
mod console_metrics_smoke;
#[cfg(feature = "console-api")]
mod feature_boundaries;
#[cfg(all(feature = "console-api", feature = "ws"))]
mod rbus_cli_smoke;
#[cfg(all(feature = "console", feature = "ws", feature = "demo-tank"))]
mod tank_session;
#[cfg(all(feature = "console-api", feature = "ws"))]
mod topic_demand;
#[cfg(all(feature = "console-api", feature = "ws"))]
mod topic_type_registry;
