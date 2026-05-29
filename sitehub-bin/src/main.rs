//! Composition root for sitehub. Wires concrete adapters into ports and
//! dispatches requests to driving adapters based on the Host header.

mod config;

use std::net::SocketAddr;

use anyhow::Context;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::signal::unix::{Signal, SignalKind, signal};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Per ADR-0034: only load .env in debug builds. Release builds (Fly.io)
    // must get config from the environment directly, never from a file that
    // could shadow Fly secrets.
    #[cfg(debug_assertions)]
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

    let shutdown_timeout = cfg.shutdown_timeout();

    let mut app = Router::new()
        .route("/api/health", get(health))
        .merge(sitehub_public_api::router())
        .merge(sitehub_admin_api::router())
        .merge(sitehub_auth_api::router());

    if cfg.request_timeout_enabled() {
        app = app.layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            cfg.request_timeout(),
        ));
    } else {
        tracing::warn!("request_timeout disabled (request_timeout_secs = 0)");
    }

    let app = app.layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new(cfg.host.parse()?, cfg.port);
    tracing::info!("sitehub listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Install signal handlers before serving so install errors propagate
    // cleanly via `?` instead of panicking from inside the async block.
    let sigterm = signal(SignalKind::terminate()).context("install SIGTERM handler")?;
    let sigint = signal(SignalKind::interrupt()).context("install SIGINT handler")?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        await_shutdown_signal(sigterm, sigint).await;
        let _ = shutdown_tx.send(());
    });

    tokio::select! {
        result = server => result?,
        () = async {
            let _ = shutdown_rx.await;
            tokio::time::sleep(shutdown_timeout).await;
            tracing::warn!(
                "shutdown timeout ({shutdown_timeout:?}) exceeded, forcing exit; in-flight requests dropped"
            );
        } => {}
    }

    Ok(())
}

async fn await_shutdown_signal(mut sigterm: Signal, mut sigint: Signal) {
    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv() => {},
    }
    tracing::info!("shutdown signal received, finishing in-flight requests");
}

async fn health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
