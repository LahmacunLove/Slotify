use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{
    models::{
        AppState,
        session::{SessionResponse, StartSessionRequest, EndSessionRequest, SessionStats},
    },
    services::SessionService,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all_sessions))
        .route("/current", get(get_current_session))
        .route("/start", post(start_session))
        .route("/end", post(end_session))
        .route("/:id", get(get_session))
        .route("/:id/download", get(get_download_link))
        .route("/statistics", get(get_session_statistics))
}

async fn get_all_sessions(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<SessionResponse>>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.get_all_sessions().await {
        Ok(sessions) => Ok(Json(sessions)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_current_session(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Option<SessionResponse>>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.get_current_session().await {
        Ok(session) => Ok(Json(session)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn start_session(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<StartSessionRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.start_session(request).await {
        Ok(session) => Ok(Json(session)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn end_session(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<EndSessionRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.end_session(&request.session_id).await {
        Ok(Some(session)) => Ok(Json(session)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_session(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.get_session_by_id(&id).await {
        Ok(Some(session)) => Ok(Json(session)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_download_link(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<String>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.get_download_link(&id).await {
        Ok(Some(link)) => Ok(Json(link)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_session_statistics(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<SessionStats>, StatusCode> {
    let session_service = SessionService::new(app_state);
    
    match session_service.get_session_statistics().await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}