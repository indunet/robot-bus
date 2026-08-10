//! Embedded Web console: static assets + monitoring API (same port as gRPC / WS).

mod api;
mod bus_publish;
mod control_plane;
mod state;
mod topic_registry;
mod topology_registry;

pub use bus_publish::StatusPublisherHandle;
pub use control_plane::ControlPlaneHandle;
pub use state::{BrokerEndpoints, ConsoleState};
pub use topic_registry::TopicTypeRegistry;
pub use topology_registry::TopologyRegistry;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::Router;
use axum::body::Body;
use axum::extract::Extension;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use rust_embed::Embed;
use tokio::net::TcpListener;

use api::{
    actions, tank_heartbeat, tank_release, tank_session, tank_status, discover, events,
    services, status, topic_info, topics, topology,
};

/// Compile-time embedded `assets/console/` (Next.js static export).
#[derive(Embed)]
#[folder = "assets/console/"]
struct Assets;

/// REST routes only (no static fallback). Uses [`Extension`] for state so this
/// `Router<()>` can merge with tonic gRPC routes.
pub fn api_router(state: Arc<ConsoleState>) -> Router {
    Router::new()
        .route("/api/v1/status", get(status))
        .route("/api/v1/discover", get(discover))
        .route("/api/v1/topics", get(topics))
        .route("/api/v1/topics/{*name}", get(topic_info))
        .route("/api/v1/topology", get(topology))
        .route("/api/v1/services", get(services))
        .route("/api/v1/actions", get(actions))
        .route("/api/v1/events", get(events))
        .route("/api/v1/tank", get(tank_status))
        .route("/api/v1/tank/session", post(tank_session))
        .route(
            "/api/v1/tank/session/{id}/heartbeat",
            post(tank_heartbeat),
        )
        .route("/api/v1/tank/session/{id}", delete(tank_release))
        .layer(Extension(state))
}

/// SPA / static asset fallback for unmatched non-gRPC paths.
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return asset_response("index.html");
    }

    // Do not SPA-fallback API, WebSocket RPC, or legacy gRPC path prefixes.
    if path.starts_with("api/") || path == "ws" || path.starts_with("robot_bus_interface.") {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    if let Some(resp) = try_asset(path) {
        return resp;
    }

    let index_path = if path.ends_with('/') {
        format!("{path}index.html")
    } else {
        format!("{path}/index.html")
    };
    if let Some(resp) = try_asset(&index_path) {
        return resp;
    }

    if let Some(resp) = try_asset("index.html") {
        return resp;
    }

    (StatusCode::NOT_FOUND, "console asset not found").into_response()
}

/// Serve console-only (no gRPC) until `shutdown` completes.
pub async fn serve_with_shutdown(
    listen: SocketAddr,
    state: Arc<ConsoleState>,
    cors_origins: Vec<String>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let mut app = api_router(state).fallback(get(static_handler));

    if !cors_origins.is_empty() {
        use axum::http::{HeaderValue, Method};
        use tower_http::cors::{AllowOrigin, CorsLayer};
        let origins: Vec<HeaderValue> = cors_origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        if !origins.is_empty() {
            let cors = CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE, header::ACCEPT, header::AUTHORIZATION]);
            app = app.layer(cors);
            log::info!("robot_bus console CORS allowlist: {cors_origins:?}");
        }
    }

    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind console HTTP on {listen}"))?;

    log::info!("robot_bus console listening on http://{listen}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .context("console HTTP server")?;
    Ok(())
}

fn try_asset(path: &str) -> Option<Response> {
    Assets::get(path).map(|file| {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .body(Body::from(file.data.into_owned()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    })
}

fn asset_response(path: &str) -> Response {
    try_asset(path)
        .unwrap_or_else(|| (StatusCode::NOT_FOUND, "console asset not found").into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_index_html() {
        assert!(
            Assets::get("index.html").is_some(),
            "assets/console/index.html must exist (run: just console)"
        );
    }
}
