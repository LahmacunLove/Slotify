use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber;

mod api;
mod models;
mod services;
mod utils;

use models::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "DJ Session Recorder API is running".to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    // Load configuration
    dotenv::dotenv().ok();
    
    // Initialize application state
    let app_state = Arc::new(AppState::new().await?);

    // Build the router
    let app = Router::new()
        .route("/health", get(health))
        .nest("/api", api::router())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("DJ Session Recorder API listening on 0.0.0.0:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}