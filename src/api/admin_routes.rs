use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    models::{
        AppState,
        dj::{DjResponse, UpdateDjRequest},
        session::B2BSessionRequest,
        lottery::LotteryStatistics,
    },
    services::{DjService, LotteryService, SessionService},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/djs", get(get_all_djs_admin))
        .route("/djs/:id", put(update_dj_admin))
        .route("/djs/:id", delete(remove_dj_admin))
        .route("/djs/:id/position", put(move_dj_position))
        .route("/queue", get(get_admin_queue))
        .route("/queue/reset", post(reset_queue))
        .route("/sessions/b2b", post(create_b2b_session))
        .route("/statistics", get(get_admin_statistics))
        .route("/timetable", get(get_timetable))
}

#[derive(Deserialize)]
struct MovePositionRequest {
    new_position: i32,
}

#[derive(Serialize)]
struct AdminQueueResponse {
    lottery_pool: Vec<DjResponse>,
    current_queue: Vec<DjResponse>,
    statistics: LotteryStatistics,
}

#[derive(Serialize)]
struct AdminStatistics {
    total_djs: usize,
    active_djs: usize,
    total_sessions: usize,
    active_sessions: usize,
    lottery_stats: LotteryStatistics,
}

#[derive(Serialize)]
struct TimetableEntry {
    position: i32,
    dj: DjResponse,
    estimated_start_time: Option<chrono::DateTime<chrono::Utc>>,
    session_type: String,
}

async fn get_all_djs_admin(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<DjResponse>>, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.get_all_djs().await {
        Ok(djs) => Ok(Json(djs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_dj_admin(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateDjRequest>,
) -> Result<Json<DjResponse>, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.update_dj(&id, request).await {
        Ok(Some(dj)) => Ok(Json(dj)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn remove_dj_admin(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let dj_service = DjService::new(app_state.clone());
    let lottery_service = LotteryService::new(app_state);
    
    // Remove from queue first if they're in it
    if let Err(_) = lottery_service.remove_from_queue(&id).await {
        // Continue anyway, DJ might not be in queue
    }
    
    match dj_service.remove_dj(&id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn move_dj_position(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<MovePositionRequest>,
) -> Result<StatusCode, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.move_dj_position(&id, request.new_position).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_admin_queue(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<AdminQueueResponse>, StatusCode> {
    let dj_service = DjService::new(app_state.clone());
    let lottery_service = LotteryService::new(app_state);
    
    let active_djs = dj_service.get_active_djs().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_queue = lottery_service.get_current_queue().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let statistics = lottery_service.get_lottery_statistics().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Filter out DJs who are already in queue from the lottery pool
    let queue_dj_ids: std::collections::HashSet<String> = current_queue.iter()
        .map(|dj| dj.id.clone())
        .collect();
    
    let lottery_pool: Vec<DjResponse> = active_djs.into_iter()
        .filter(|dj| !queue_dj_ids.contains(&dj.id))
        .collect();

    Ok(Json(AdminQueueResponse {
        lottery_pool,
        current_queue,
        statistics,
    }))
}

async fn reset_queue(
    State(app_state): State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> {
    let lottery_service = LotteryService::new(app_state);
    
    match lottery_service.reset_lottery().await {
        Ok(()) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_b2b_session(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<B2BSessionRequest>,
) -> Result<StatusCode, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.create_b2b_session(request).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_admin_statistics(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<AdminStatistics>, StatusCode> {
    let dj_service = DjService::new(app_state.clone());
    let lottery_service = LotteryService::new(app_state.clone());
    let session_service = SessionService::new(app_state);
    
    let all_djs = dj_service.get_all_djs().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_djs = dj_service.get_active_djs().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let session_stats = session_service.get_session_statistics().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let lottery_stats = lottery_service.get_lottery_statistics().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminStatistics {
        total_djs: all_djs.len(),
        active_djs: active_djs.len(),
        total_sessions: session_stats.total_sessions,
        active_sessions: session_stats.active_sessions,
        lottery_stats,
    }))
}

async fn get_timetable(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<TimetableEntry>>, StatusCode> {
    let lottery_service = LotteryService::new(app_state.clone());
    let session_service = SessionService::new(app_state);
    
    let current_queue = lottery_service.get_current_queue().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut timetable = Vec::new();
    let mut estimated_time = chrono::Utc::now();
    
    // Add current session if any
    if let Ok(Some(current_session)) = session_service.get_current_session().await {
        // Current session is already running, start estimates from when it might end
        estimated_time = estimated_time + chrono::Duration::minutes(60); // Assume 60 min default
    }
    
    for (i, dj) in current_queue.iter().enumerate() {
        timetable.push(TimetableEntry {
            position: (i + 1) as i32,
            dj: dj.clone(),
            estimated_start_time: Some(estimated_time),
            session_type: "solo".to_string(),
        });
        
        // Add estimated duration for next DJ (60 minutes default)
        estimated_time = estimated_time + chrono::Duration::minutes(60);
    }

    Ok(Json(timetable))
}