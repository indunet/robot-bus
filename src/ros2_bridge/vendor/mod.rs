//! Typed ROS IDL bindings used by the in-process bridge.
//!
//! rclrs vendors `example_interfaces` services with matching field layouts, but
//! a different type name. Real ROS graphs use `std_srvs`, so we vendor those
//! here and link against system `libstd_srvs__rosidl_*`.

#![allow(non_camel_case_types)]

pub mod std_srvs;
