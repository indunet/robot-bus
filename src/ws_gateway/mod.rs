//! Multiplexed WebSocket RPC gateway over the ZeroMQ buses.
//!
//! Enabled with `--features ws`. Covers message Subscribe / Publish, service
//! Call, and action SendGoal (one GOAL request → FEEDBACK / RESULT stream).
//! Native and browser clients share `/ws` (V2: one WebSocket, many streams).

pub mod action;
pub mod message;
pub mod rpc_status;
pub mod server;
pub mod service;
pub mod sub_demux;
pub mod ws;
pub mod ws_frame;

pub mod pb {
    // Messages-only (no tonic). Regenerated via `just gen-rust` / scripts/gen_rust_msgs
    // is switched to prost-only for gateway protos.
    include!("../generated/robot_bus_interfaces/grpc/v1/messages.inc.rs");
}

pub use action::ActionGatewayService;
pub use message::MessageGatewayService;
pub use server::{GatewayConfig, serve, serve_on_listener, serve_with_shutdown};
pub use service::ServiceGatewayService;
