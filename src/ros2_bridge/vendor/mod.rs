//! Typed ROS IDL bindings used by the in-process bridge.
//!
//! `std_srvs` is vendored here and links system `libstd_srvs__rosidl_*`
//! (C typesupport, not rust IDL). `example_interfaces` Fibonacci uses
//! `ros_env` overlay re-exports (see `ros_idl`).

#![allow(non_camel_case_types)]

pub mod std_srvs;
