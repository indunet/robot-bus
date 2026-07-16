//! HTTP server: native gRPC + gRPC-Web on one port.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use http::Method;
use http::header::HeaderName;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use super::message::MessageGatewayService;
use super::pb::message_gateway_server::MessageGatewayServer;

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub listen: SocketAddr,
    pub message_xpub: String,
    /// When empty, allow any origin (local-dev default).
    pub cors_origins: Vec<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:15770".parse().expect("default listen"),
            message_xpub: crate::transports::message_xpub_endpoint("127.0.0.1", "tcp")
                .unwrap_or_else(|_| "tcp://127.0.0.1:15561".to_string()),
            cors_origins: Vec::new(),
        }
    }
}

pub async fn serve(config: GatewayConfig) -> Result<()> {
    let service = MessageGatewayService::new(config.message_xpub.clone());
    let cors = build_cors(&config.cors_origins)?;

    log::info!(
        "robot_bus_grpc_gateway listening on http://{} (gRPC + gRPC-Web); message XPUB {}",
        config.listen,
        config.message_xpub
    );

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .add_service(MessageGatewayServer::new(service))
        .serve(config.listen)
        .await
        .context("gateway server")?;
    Ok(())
}

fn build_cors(origins: &[String]) -> Result<CorsLayer> {
    // grpc-web clients need these headers exposed / allowed.
    let grpc_headers = [
        HeaderName::from_static("content-type"),
        HeaderName::from_static("x-grpc-web"),
        HeaderName::from_static("x-user-agent"),
        HeaderName::from_static("grpc-timeout"),
        HeaderName::from_static("grpc-status"),
        HeaderName::from_static("grpc-message"),
        HeaderName::from_static("grpc-encoding"),
        HeaderName::from_static("grpc-accept-encoding"),
    ];

    if origins.is_empty() {
        Ok(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any)
            .expose_headers(grpc_headers))
    } else {
        let parsed = origins
            .iter()
            .map(|o| o.parse().context("invalid --cors-origin"))
            .collect::<Result<Vec<_>>>()?;
        Ok(CorsLayer::new()
            .allow_origin(AllowOrigin::list(parsed))
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any)
            .expose_headers(grpc_headers))
    }
}
