//! WebSocket communication: multiplexed RPC, framing, and server-side bus integration.
//!
//! Enabled with `--features ws`. Covers message Subscribe / Publish, service
//! Call, and action SendGoal (one GOAL request → FEEDBACK / RESULT stream).
//! Native and browser clients share `/ws-rpc` (V3: one WebSocket, many streams).

pub mod action;
pub mod message;
pub mod rpc_status;
pub mod server;
pub mod service;
pub mod sub_demux;
pub mod subscription_queue;
pub mod ws;
pub mod ws_frame;

pub use action::WsActionHandler;
pub use message::WsMessageService;
pub use server::{WsServerConfig, serve, serve_on_listener, serve_with_shutdown};
pub use service::WsServiceHandler;
