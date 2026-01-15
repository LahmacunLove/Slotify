use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents the overall event session (the entire DJ night)
/// This is different from individual DJ sessions - it manages the whole event flow
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventSession {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub slot_duration_minutes: i32,
    pub late_arrival_cutoff_hours: i32, // Hours after start when penalty kicks in
    pub is_active: bool,
    pub current_dj_id: Option<String>,
    pub current_slot_started_at: Option<DateTime<Utc>>,
    pub next_draw_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartEventRequest {
    pub slot_duration_minutes: Option<i32>, // Default to 60 if not provided
    pub late_arrival_cutoff_hours: Option<i32>, // Default to 2 if not provided
    pub started_at: Option<DateTime<Utc>>, // Optional custom start time
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventSessionResponse {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub slot_duration_minutes: i32,
    pub late_arrival_cutoff_hours: i32,
    pub is_active: bool,
    pub current_dj_id: Option<String>,
    pub current_dj_name: Option<String>,
    pub current_slot_started_at: Option<DateTime<Utc>>,
    pub next_draw_at: Option<DateTime<Utc>>,
    pub elapsed_minutes: i32,
    pub current_slot_progress_percent: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimetableEntry {
    pub position: i32,
    pub dj_id: String,
    pub dj_name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub status: TimetableEntryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimetableEntryStatus {
    Completed,
    InProgress,
    Upcoming,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timetable {
    pub event_id: String,
    pub event_started_at: DateTime<Utc>,
    pub entries: Vec<TimetableEntry>,
    pub total_djs: usize,
    pub completed_sets: usize,
}

impl EventSession {
    pub fn new(slot_duration_minutes: i32, late_arrival_cutoff_hours: i32, started_at: Option<DateTime<Utc>>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            started_at: started_at.unwrap_or_else(|| Utc::now()),
            ended_at: None,
            slot_duration_minutes,
            late_arrival_cutoff_hours,
            is_active: true,
            current_dj_id: None,
            current_slot_started_at: None,
            next_draw_at: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active && self.ended_at.is_none()
    }

    pub fn elapsed_minutes(&self) -> i32 {
        Utc::now()
            .signed_duration_since(self.started_at)
            .num_minutes() as i32
    }

    pub fn should_apply_late_penalty(&self, registration_time: DateTime<Utc>) -> bool {
        let hours_since_start = registration_time
            .signed_duration_since(self.started_at)
            .num_hours();
        hours_since_start > self.late_arrival_cutoff_hours as i64
    }

    pub fn current_slot_progress_percent(&self) -> Option<f32> {
        if let Some(slot_start) = self.current_slot_started_at {
            let elapsed = Utc::now()
                .signed_duration_since(slot_start)
                .num_minutes() as f32;
            let progress = (elapsed / self.slot_duration_minutes as f32) * 100.0;
            Some(progress.min(100.0))
        } else {
            None
        }
    }

    pub fn should_draw_next(&self) -> bool {
        if let Some(next_draw) = self.next_draw_at {
            Utc::now() >= next_draw
        } else {
            false
        }
    }

    pub fn calculate_next_draw_time(&self, slot_start: DateTime<Utc>) -> DateTime<Utc> {
        // Draw at 50% of the slot duration
        let half_duration = chrono::Duration::minutes(self.slot_duration_minutes as i64 / 2);
        slot_start + half_duration
    }
}
