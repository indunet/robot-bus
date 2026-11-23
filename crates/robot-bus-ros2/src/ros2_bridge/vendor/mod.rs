//! Typed ROS IDL bindings used by the in-process bridge.
//!
//! `std_srvs` is vendored here and links system `libstd_srvs__rosidl_*`
//! (C typesupport). `example_interfaces` Fibonacci uses `ros_env` re-exports.

#![allow(non_camel_case_types)]

pub mod std_srvs;
