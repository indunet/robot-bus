//! C ABI for robot-bus (opaque handles). Mirrored from Python / napi surfaces.

mod ffi;
mod pub_sub;
mod clients;
mod context;
mod node;
mod node_entities;
mod node_params;
mod executor;
mod broker;
mod ros2_bridge;
mod tf;

// Re-export helpers used by `ros2_bridge` (including the `ros2` feature path).
#[allow(unused_imports)]
pub(crate) use ffi::{bus_err, clear_error, cstr_opt, cstr_req, err, ok, set_error};
