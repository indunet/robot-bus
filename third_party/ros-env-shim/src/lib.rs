//! Overlay re-export (`AMENT_PREFIX_PATH`) plus field-complete stubs for `use_ros_shim`.
//!
//! crates.io `ros-env` 0.2 empties `interfaces.rs` under `use_ros_shim`, but
//! robot-bus mappers still construct `ros_env::*` message fields. This crate patches that.

#![allow(dead_code, missing_docs, non_camel_case_types, unused_imports)]

#[cfg(feature = "use_ros_shim")]
mod shim;

#[cfg(feature = "use_ros_shim")]
pub use shim::*;

#[cfg(not(feature = "use_ros_shim"))]
include!(concat!(env!("OUT_DIR"), "/interfaces.rs"));
