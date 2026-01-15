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
use services::{EventService, LotteryService};

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

async fn auto_draw_background_task(app_state: Arc<AppState>) {
    info!("Starting automatic draw background task");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        interval.tick().await;

        let event_service = EventService::new(app_state.clone());
        let lottery_service = LotteryService::new(app_state.clone());

        match event_service.check_and_trigger_auto_draw().await {
            Ok(should_draw) => {
                if should_draw {
                    info!("Automatic draw triggered - drawing next DJ");
                    match lottery_service.draw_next_dj().await {
                        Ok(Some(draw)) => {
                            info!("Successfully drew next DJ: {}", draw.winner.name);
                        }
                        Ok(None) => {
                            info!("No eligible DJs to draw");
                        }
                        Err(e) => {
                            tracing::error!("Error drawing next DJ: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error checking auto-draw trigger: {}", e);
            }
        }
    }
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

    // Spawn background task for automatic lottery draws
    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        auto_draw_background_task(app_state_clone).await;
    });

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