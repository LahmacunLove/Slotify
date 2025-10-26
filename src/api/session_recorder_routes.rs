use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    models::AppState,
    services::{SessionService, RecorderSession},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/available-sessions", get(get_available_recorder_sessions))
        .route("/link/:session_id/:recorder_id/:recorder_session_id", post(link_session_to_recorder))
        .route("/auto-link/:session_id", post(auto_link_session))
        .route("/download/:session_id/:format", get(get_recording_download_url))
        .route("/session/:session_id", get(get_session_with_recorder_info))
}

#[derive(Deserialize)]
struct AutoLinkQuery {
    tolerance_minutes: Option<i64>,
}

async fn get_available_recorder_sessions(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<Vec<RecorderSession>>, StatusCode> {
    let session_service = match SessionService::new_with_recorder(app_state).await {
        Ok(service) => service,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };
    
    match session_service.get_available_recorder_sessions().await {
        Ok(sessions) => Ok(Json(sessions)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn link_session_to_recorder(
    State(app_state): State<Arc<AppState>>,
    Path((session_id, recorder_id, recorder_session_id)): Path<(String, String, String)>,
) -> Result<StatusCode, StatusCode> {
    let session_service = match SessionService::new_with_recorder(app_state).await {
        Ok(service) => service,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };
    
    match session_service.link_to_recorder_session(&session_id, &recorder_session_id, &recorder_id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn auto_link_session(
    State(app_state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Query(params): Query<AutoLinkQuery>,
) -> Result<Json<AutoLinkResponse>, StatusCode> {
    let session_service = match SessionService::new_with_recorder(app_state.clone()).await {
        Ok(service) => service,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };

    // Get the session start time
    let session = match session_service.get_session_by_id(&session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let session_start = chrono::DateTime::parse_from_rfc3339(&session.started_at.to_rfc3339())
        .unwrap()
        .with_timezone(&chrono::Utc);

    match session_service.auto_link_recorder_session(&session_id, session_start).await {
        Ok(linked) => Ok(Json(AutoLinkResponse { 
            success: linked,
            message: if linked { 
                "Session successfully auto-linked to recorder session".to_string() 
            } else { 
                "No matching recorder session found".to_string() 
            }
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_recording_download_url(
    State(app_state): State<Arc<AppState>>,
    Path((session_id, format)): Path<(String, String)>,
) -> Result<Json<DownloadUrlResponse>, StatusCode> {
    let session_service = match SessionService::new_with_recorder(app_state).await {
        Ok(service) => service,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };
    
    match session_service.get_recording_download_url(&session_id, &format).await {
        Ok(Some(url)) => Ok(Json(DownloadUrlResponse { 
            url: Some(url),
            expires_in_seconds: 3600,
            format,
        })),
        Ok(None) => Ok(Json(DownloadUrlResponse { 
            url: None,
            expires_in_seconds: 0,
            format,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_session_with_recorder_info(
    State(app_state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionWithRecorderInfo>, StatusCode> {
    let session_service = match SessionService::new_with_recorder(app_state).await {
        Ok(service) => service,
        Err(_) => return Err(StatusCode::SERVICE_UNAVAILABLE),
    };
    
    match session_service.get_session_with_recorder_info(&session_id).await {
        Ok(Some(session)) => {
            // Get available download URLs
            let ogg_url = session_service.get_recording_download_url(&session_id, "ogg").await.ok().flatten();
            let flac_url = session_service.get_recording_download_url(&session_id, "flac").await.ok().flatten();
            let waveform_url = session_service.get_recording_download_url(&session_id, "waveform").await.ok().flatten();

            Ok(Json(SessionWithRecorderInfo {
                session,
                recording_urls: RecordingUrls {
                    ogg_url,
                    flac_url,
                    waveform_url,
                },
            }))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Serialize)]
struct AutoLinkResponse {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct DownloadUrlResponse {
    url: Option<String>,
    expires_in_seconds: u64,
    format: String,
}

#[derive(Serialize)]
struct SessionWithRecorderInfo {
    #[serde(flatten)]
    session: crate::models::session::SessionResponse,
    recording_urls: RecordingUrls,
}

#[derive(Serialize)]
struct RecordingUrls {
    ogg_url: Option<String>,
    flac_url: Option<String>,
    waveform_url: Option<String>,
}