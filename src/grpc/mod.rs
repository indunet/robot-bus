//! gRPC / gRPC-Web gateway over the ZeroMQ buses.
//!
//! Enabled with `--features grpc`. Covers message Subscribe, service Call,
//! and action bidirectional Run (GOAL / CANCEL ↔ FEEDBACK / RESULT).

pub mod action;
pub mod message;
pub mod server;
pub mod service;

pub mod pb {
    tonic::include_proto!("robot_bus.grpc.v1");
}

pub use action::ActionGatewayService;
pub use message::MessageGatewayService;
pub use server::{serve, GatewayConfig};
pub use service::ServiceGatewayService;
