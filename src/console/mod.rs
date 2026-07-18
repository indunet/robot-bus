//! Embedded Web console: static assets served over HTTP (default `:15771`).

use std::future::Future;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::Router;
use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use rust_embed::Embed;
use tokio::net::TcpListener;

/// Compile-time embedded `assets/console/` (Next.js static export).
#[derive(Embed)]
#[folder = "assets/console/"]
struct Assets;

/// Serve until `shutdown` completes.
pub async fn serve_with_shutdown(
    listen: SocketAddr,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let app = Router::new().fallback(get(static_handler));
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

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return asset_response("index.html");
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

    // SPA-style fallback for client-side routes.
    if let Some(resp) = try_asset("index.html") {
        return resp;
    }

    (StatusCode::NOT_FOUND, "console asset not found").into_response()
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
    try_asset(path).unwrap_or_else(|| {
        (StatusCode::NOT_FOUND, "console asset not found").into_response()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_index_html() {
        assert!(
            Assets::get("index.html").is_some(),
            "assets/console/index.html must exist (run: cd console && pnpm build && ../scripts/sync_console_assets.sh)"
        );
    }
}
