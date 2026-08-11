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
