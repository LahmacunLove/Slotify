use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dj {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub weight: f64,
    pub is_active: bool,
    pub position_in_queue: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDjRequest {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateDjRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub weight: Option<f64>,
    pub is_active: Option<bool>,
    pub position_in_queue: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjResponse {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub weight: f64,
    pub is_active: bool,
    pub position_in_queue: Option<i32>,
    pub estimated_time: Option<DateTime<Utc>>,
}

impl From<Dj> for DjResponse {
    fn from(dj: Dj) -> Self {
        Self {
            id: dj.id,
            name: dj.name,
            email: dj.email,
            registered_at: dj.registered_at,
            weight: dj.weight,
            is_active: dj.is_active,
            position_in_queue: dj.position_in_queue,
            estimated_time: None, // This will be calculated based on current queue
        }
    }
}

impl Dj {
    pub fn new(name: String, email: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            registered_at: Utc::now(),
            weight: 1.0,
            is_active: true,
            position_in_queue: None,
        }
    }

    pub fn calculate_weight(&self, late_arrival_penalty: f64) -> f64 {
        let hours_since_registration = Utc::now()
            .signed_duration_since(self.registered_at)
            .num_hours() as f64;
        
        // Apply penalty for late arrivals (after midnight or specific time)
        let current_hour = Utc::now().hour();
        let penalty = if current_hour >= 24 || hours_since_registration > 4.0 {
            late_arrival_penalty
        } else {
            1.0
        };

        self.weight * penalty
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DjPool {
    pub active_djs: Vec<DjResponse>,
    pub current_dj: Option<DjResponse>,
    pub next_dj: Option<DjResponse>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GuestRequest {
    pub guest_name: String,
    pub guest_email: String,
    pub message: Option<String>,
    pub target_dj_id: String,
}