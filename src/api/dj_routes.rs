use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use std::sync::Arc;

use crate::{
    models::{
        AppState,
        dj::{CreateDjRequest, UpdateDjRequest, DjResponse, DjPool, GuestRequest},
    },
    services::DjService,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all_djs))
        .route("/register", post(register_dj))
        .route("/pool", get(get_dj_pool))
        .route("/:id", get(get_dj))
        .route("/:id", put(update_dj))
        .route("/:id", delete(remove_dj))
        .route("/:id/request", post(submit_guest_request))
}

async fn get_all_djs(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<DjResponse>>, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.get_all_djs().await {
        Ok(djs) => Ok(Json(djs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn register_dj(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<CreateDjRequest>,
) -> Result<Json<DjResponse>, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.register_dj(request).await {
        Ok(dj) => Ok(Json(dj)),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_dj_pool(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<DjPool>, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.get_dj_pool().await {
        Ok(pool) => Ok(Json(pool)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_dj(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<DjResponse>, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.get_dj_by_id(&id).await {
        Ok(Some(dj)) => Ok(Json(dj)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_dj(
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

async fn remove_dj(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.remove_dj(&id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn submit_guest_request(
    State(app_state): State<Arc<AppState>>,
    Path(dj_id): Path<String>,
    Json(request): Json<GuestRequest>,
) -> Result<StatusCode, StatusCode> {
    let dj_service = DjService::new(app_state);
    
    match dj_service.submit_guest_request(dj_id, request).await {
        Ok(()) => Ok(StatusCode::CREATED),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}