//! Composition root for sitehub. Wires concrete adapters into ports and
//! dispatches requests to driving adapters based on the Host header.

use std::net::SocketAddr;

use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::{Value, json};
use tower_http::trace::TraceLayer;

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

    let host = std::env::var("SITEHUB_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("SITEHUB_PORT")
        .unwrap_or_else(|_| "3000".into())
        .parse()?;

    let app = Router::new()
        .route("/api/health", get(health))
        .merge(sitehub_public_api::router())
        .merge(sitehub_admin_api::router())
        .merge(sitehub_auth_api::router())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new(host.parse()?, port);
    tracing::info!("sitehub listening on {addr}");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}
