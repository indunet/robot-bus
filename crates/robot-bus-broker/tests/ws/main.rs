//! WebSocket RPC and node integration tests.

#[path = "../common/mod.rs"]
mod common;

#[cfg(any(feature = "ws", feature = "console-api"))]
mod api_port_fallback;
#[cfg(feature = "ws")]
mod ws_node;
#[cfg(feature = "ws")]
mod ws_rpc;
