//! Project-local bus messages for `my_pkg.srv.v1.AddTwoInts`.
//!
//! Mirrors [`examples/ros2_bridge/proto/my_pkg/srv/v1/add_two_ints.proto`],
//! which pairs with the ROS definition
//! [`examples/ros2_bridge/ros2/my_pkg/srv/AddTwoInts.srv`].
//!
//! An app crate would normally `prost_build` the `.proto` in `build.rs`; this
//! example keeps the prost structs inline so `cargo run --example` needs no
//! extra codegen step.

use prost::Message;

/// `my_pkg.srv.v1.AddTwoIntsRequest`
#[derive(Clone, PartialEq, Message)]
pub struct AddTwoIntsRequest {
    #[prost(int64, tag = "1")]
    pub a: i64,
    #[prost(int64, tag = "2")]
    pub b: i64,
}

/// `my_pkg.srv.v1.AddTwoIntsResponse`
#[derive(Clone, PartialEq, Message)]
pub struct AddTwoIntsResponse {
    #[prost(int64, tag = "1")]
    pub sum: i64,
}
