//! HTTP server: multiplexed WebSocket RPC (+ optional console) on one port.

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use axum::routing::get;
use http::Method;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use super::action::ActionGatewayService;
use super::message::MessageGatewayService;
use super::service::ServiceGatewayService;
use super::ws::{WsGatewayState, ws_upgrade};

#[cfg(feature = "console")]
use crate::console::{self, ConsoleState};
use crate::discovery::DiscoverResponse;

#[derive(Clone)]
pub struct GatewayConfig {
    pub listen: SocketAddr,
    pub message_xpub: String,
    pub message_xsub: String,
    pub service_frontend: String,
    pub action_frontend: String,
    /// When empty, allow any origin (local-dev default).
    pub cors_origins: Vec<String>,
    /// Broker endpoint map for `GET /api/v1/discover` (always served when set).
    pub discover: Option<Arc<DiscoverResponse>>,
    /// When set (feature `console`), serve REST + static UI on the same listener.
    #[cfg(feature = "console")]
    pub console: Option<Arc<ConsoleState>>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen: "0.0.0.0:15570".parse().expect("default listen"),
            message_xpub: "tcp://127.0.0.1:15561".to_string(),
            message_xsub: "tcp://127.0.0.1:15560".to_string(),
            service_frontend: "tcp://127.0.0.1:15662".to_string(),
            action_frontend: "tcp://127.0.0.1:15664".to_string(),
            cors_origins: Vec::new(),
            discover: None,
            #[cfg(feature = "console")]
            console: None,
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
        .with_context(|| format!("bind API listen {listen}"))?;
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

    #[cfg(feature = "console")]
    let with_console = config.console.is_some();
    #[cfg(not(feature = "console"))]
    let with_console = false;

    log::info!(
        "robot_bus WebSocket RPC gateway listening on http://{} (/ws{}); \
         message XPUB {}; message XSUB {}; service frontend {}; action frontend {}",
        config.listen,
        if with_console { " + console" } else { "" },
        config.message_xpub,
        config.message_xsub,
        config.service_frontend,
        config.action_frontend
    );

    let ws_state = Arc::new(WsGatewayState {
        message,
        service,
        action,
    });

    let app = Router::new()
        .route("/ws", get(ws_upgrade))
        .with_state(ws_state);

    let app = if let Some(disc) = config.discover.clone() {
        app.route(
            "/api/v1/discover",
            get({
                let disc = disc;
                move || async move { axum::Json((*disc).clone()) }
            }),
        )
    } else {
        app
    };

    #[cfg(feature = "console")]
    let app = match config.console {
        Some(state) => app
            .merge(console::api_router(state))
            .fallback(get(console::static_handler)),
        None => app,
    };

    let app = app.layer(cors);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .context("gateway server")?;
    Ok(())
}

fn build_cors(origins: &[String]) -> Result<CorsLayer> {
    if origins.is_empty() {
        Ok(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any))
    } else {
        let parsed = origins
            .iter()
            .map(|o| o.parse().context("invalid --cors-origin"))
            .collect::<Result<Vec<_>>>()?;
        Ok(CorsLayer::new()
            .allow_origin(AllowOrigin::list(parsed))
            .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any))
    }
}
