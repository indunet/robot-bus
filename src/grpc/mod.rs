//! gRPC / gRPC-Web gateway over the ZeroMQ buses.
//!
//! Enabled with `--features grpc`. First release covers message Subscribe;
//! service / action gateways can land as sibling modules later.

pub mod message;
pub mod server;

pub mod pb {
    tonic::include_proto!("robot_bus.grpc.v1");
}

pub use message::MessageGatewayService;
pub use server::{serve, GatewayConfig};
