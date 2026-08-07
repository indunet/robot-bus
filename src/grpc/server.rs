//! HTTP server: native gRPC + gRPC-Web on one port.

use std::future::Future;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use http::Method;
use http::header::HeaderName;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use super::action::ActionGatewayService;
use super::message::MessageGatewayService;
use super::pb::action_gateway_server::ActionGatewayServer;
use super::pb::message_gateway_server::MessageGatewayServer;
use super::pb::service_gateway_server::ServiceGatewayServer;
use super::service::ServiceGatewayService;

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub listen: SocketAddr,
    pub message_xpub: String,
    pub message_xsub: String,
    pub service_frontend: String,
    pub action_frontend: String,
    /// When empty, allow any origin (local-dev default).
    pub cors_origins: Vec<String>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:15770".parse().expect("default listen"),
            message_xpub: crate::transports::message_xpub_endpoint("127.0.0.1", "tcp")
                .unwrap_or_else(|_| "tcp://127.0.0.1:15561".to_string()),
            message_xsub: crate::transports::message_xsub_endpoint("127.0.0.1", "tcp")
                .unwrap_or_else(|_| "tcp://127.0.0.1:15560".to_string()),
            service_frontend: crate::transports::service_frontend_endpoint("127.0.0.1", "tcp")
                .unwrap_or_else(|_| "tcp://127.0.0.1:15662".to_string()),
            action_frontend: crate::transports::action_frontend_endpoint("127.0.0.1", "tcp")
                .unwrap_or_else(|_| "tcp://127.0.0.1:15664".to_string()),
            cors_origins: Vec::new(),
        }
    }
}

pub async fn serve(config: GatewayConfig) -> Result<()> {
    serve_with_shutdown(config, std::future::pending::<()>()).await
}

/// Serve until `shutdown` completes, then drain and exit.
pub async fn serve_with_shutdown(
    config: GatewayConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let listen = config.listen;
    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind gRPC listen {listen}"))?;
    serve_on_listener(config, listener, shutdown).await
}

/// Serve on an already-bound listener (caller owns the bind / fail-fast).
pub async fn serve_on_listener(
    config: GatewayConfig,
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let message =
        MessageGatewayService::new(config.message_xpub.clone(), config.message_xsub.clone());
    let service = ServiceGatewayService::new(config.service_frontend.clone());
    let action = ActionGatewayService::new(config.action_frontend.clone());
    let cors = build_cors(&config.cors_origins)?;

    log::info!(
        "robot_bus gRPC gateway listening on http://{} (gRPC + gRPC-Web); \
         message XPUB {}; message XSUB {}; service frontend {}; action frontend {}",
        config.listen,
        config.message_xpub,
        config.message_xsub,
        config.service_frontend,
        config.action_frontend
    );

    // TcpListenerStream alone leaves TCP_NODELAY off; with serve_with_incoming_*
    // tonic's Server::tcp_nodelay() is ignored — wrap so small streaming frames
    // are not delayed by Nagle (~40ms classic delayed-ACK stall on action streams).
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .add_service(MessageGatewayServer::new(message))
        .add_service(ServiceGatewayServer::new(service))
        .add_service(ActionGatewayServer::new(action))
        .serve_with_incoming_shutdown(incoming, shutdown)
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
