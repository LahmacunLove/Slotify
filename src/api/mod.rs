use axum::{routing::get, Router};
use std::sync::Arc;

use crate::models::AppState;

mod dj_routes;
mod session_routes;
mod lottery_routes;
mod admin_routes;
mod session_recorder_routes;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/djs", dj_routes::router())
        .nest("/sessions", session_routes::router())
        .nest("/lottery", lottery_routes::router())
        .nest("/admin", admin_routes::router())
        .nest("/session-recorder", session_recorder_routes::router())
}