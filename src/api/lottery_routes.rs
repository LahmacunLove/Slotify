use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{
    models::{
        AppState,
        lottery::{LotteryDraw, LotteryStatistics},
        dj::DjResponse,
    },
    services::LotteryService,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/draw", post(draw_next_dj))
        .route("/queue", get(get_current_queue))
        .route("/next", get(get_next_dj))
        .route("/statistics", get(get_lottery_statistics))
        .route("/reset", post(reset_lottery))
}

async fn draw_next_dj(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Option<LotteryDraw>>, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.draw_next_dj().await {
        Ok(draw) => Ok(Json(draw)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_current_queue(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<DjResponse>>, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.get_current_queue().await {
        Ok(queue) => Ok(Json(queue)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_next_dj(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Option<DjResponse>>, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.get_next_dj().await {
        Ok(next_dj) => Ok(Json(next_dj)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_lottery_statistics(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<LotteryStatistics>, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.get_lottery_statistics().await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn reset_lottery(
    State(app_state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.reset_lottery().await {
        Ok(()) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}