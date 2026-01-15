use crate::models::{AppState, event_session::StartEventRequest};
use crate::services::EventService;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/start", post(start_event))
        .route("/current", get(get_current_event))
        .route("/end", post(end_event))
        .route("/timetable", get(get_timetable))
}

async fn start_event(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<StartEventRequest>,
) -> impl IntoResponse {
    let service = EventService::new(app_state);

    match service.start_event(request).await {
        Ok(event) => (StatusCode::OK, Json(event)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn get_current_event(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let service = EventService::new(app_state);

    match service.get_active_event_response().await {
        Ok(Some(event)) => (StatusCode::OK, Json(event)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "No active event").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn end_event(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let service = EventService::new(app_state);

    match service.end_event().await {
        Ok(event) => (StatusCode::OK, Json(event)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

async fn get_timetable(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    let service = EventService::new(app_state);

    match service.get_timetable().await {
        Ok(Some(timetable)) => (StatusCode::OK, Json(timetable)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "No active event").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
