//! Composition root for sitehub. Wires concrete adapters into ports and
//! dispatches requests to driving adapters based on the Host header.

mod config;

use std::net::SocketAddr;

use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match dotenvy::dotenv() {
        Ok(_) | Err(dotenvy::Error::Io(_)) => {}
        Err(e) => eprintln!("warning: malformed .env file: {e}"),
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sitehub=debug".into()),
        )
        .init();

    let cfg = Config::load()?;
    tracing::info!(?cfg, "loaded config");

    let request_timeout = cfg.request_timeout();
    let shutdown_timeout = cfg.shutdown_timeout();

    let app = Router::new()
        .route("/api/health", get(health))
        .merge(sitehub_public_api::router())
        .merge(sitehub_admin_api::router())
        .merge(sitehub_auth_api::router())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new(cfg.host.parse()?, cfg.port);
    tracing::info!("sitehub listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());

    match tokio::time::timeout(shutdown_timeout, server).await {
        Ok(result) => result?,
        Err(_) => {
            tracing::warn!(
                "shutdown timeout ({shutdown_timeout:?}) exceeded, forcing exit; in-flight requests dropped"
            );
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv() => {},
    }

    tracing::info!("shutdown signal received, finishing in-flight requests");
}

async fn health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
