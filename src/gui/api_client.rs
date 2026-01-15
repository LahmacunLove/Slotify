use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjResponse {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub registered_at: String,
    pub weight: f64,
    pub is_active: bool,
    pub position_in_queue: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryDrawResponse {
    pub winner: DjResponse,
    pub participants: Vec<Value>,
    pub drawn_at: String,
    pub algorithm_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub dj_id: String,
    pub dj_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_minutes: Option<i32>,
    pub upload_status: String,
    pub session_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDjRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSessionRequest {
    pub dj_id: String,
    pub session_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjPoolResponse {
    pub active_djs: Vec<DjResponse>,
    pub current_dj: Option<DjResponse>,
    pub next_dj: Option<DjResponse>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSessionResponse {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub slot_duration_minutes: i32,
    pub late_arrival_cutoff_hours: i32,
    pub is_active: bool,
    pub current_dj_id: Option<String>,
    pub current_dj_name: Option<String>,
    pub current_slot_started_at: Option<String>,
    pub next_draw_at: Option<String>,
    pub elapsed_minutes: i32,
    pub current_slot_progress_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartEventRequest {
    pub slot_duration_minutes: Option<i32>,
    pub late_arrival_cutoff_hours: Option<i32>,
    pub started_at: Option<String>, // ISO 8601 datetime string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimetableEntry {
    pub position: i32,
    pub dj_id: String,
    pub dj_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_minutes: Option<i32>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timetable {
    pub event_id: String,
    pub event_started_at: String,
    pub entries: Vec<TimetableEntry>,
    pub total_djs: usize,
    pub completed_sets: usize,
}

pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    // DJ Endpoints
    pub fn get_all_djs(&self) -> Result<Vec<DjResponse>, String> {
        let url = format!("{}/djs", self.base_url);
        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<Vec<DjResponse>>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn register_dj(&self, name: String, email: String) -> Result<DjResponse, String> {
        let url = format!("{}/djs/register", self.base_url);
        let request = RegisterDjRequest { name, email };

        self.client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<DjResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn delete_dj(&self, dj_id: &str) -> Result<(), String> {
        let url = format!("{}/djs/{}", self.base_url, dj_id);

        self.client
            .delete(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        Ok(())
    }

    pub fn get_dj_pool(&self) -> Result<DjPoolResponse, String> {
        let url = format!("{}/djs/pool", self.base_url);

        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<DjPoolResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // Lottery Endpoints
    pub fn draw_next_dj(&self) -> Result<LotteryDrawResponse, String> {
        let url = format!("{}/lottery/draw", self.base_url);

        self.client
            .post(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<LotteryDrawResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn get_lottery_queue(&self) -> Result<Vec<DjResponse>, String> {
        let url = format!("{}/lottery/queue", self.base_url);

        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<Vec<DjResponse>>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn get_lottery_statistics(&self) -> Result<Value, String> {
        let url = format!("{}/lottery/statistics", self.base_url);

        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<Value>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn reset_lottery(&self) -> Result<(), String> {
        let url = format!("{}/lottery/reset", self.base_url);

        self.client
            .post(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        Ok(())
    }

    pub fn clear_all_data(&self) -> Result<(), String> {
        // End event if running (ignore error if no event)
        let _ = self.end_event();

        // Reset lottery queue (ignore error if nothing to reset)
        let _ = self.reset_lottery();

        // Delete all DJs - this is critical, so check for errors
        let djs = self.get_all_djs()
            .map_err(|e| format!("Failed to get DJs list: {}", e))?;

        let mut failed_deletions = Vec::new();
        for dj in djs {
            if let Err(e) = self.delete_dj(&dj.id) {
                failed_deletions.push(format!("{}: {}", dj.name, e));
            }
        }

        if !failed_deletions.is_empty() {
            return Err(format!("Failed to delete some DJs: {}", failed_deletions.join(", ")));
        }

        Ok(())
    }

    // Session Endpoints
    pub fn start_session(&self, dj_id: String, session_type: String) -> Result<SessionResponse, String> {
        let url = format!("{}/sessions/start", self.base_url);
        let request = StartSessionRequest { dj_id, session_type };

        self.client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<SessionResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn end_session(&self, session_id: &str) -> Result<SessionResponse, String> {
        let url = format!("{}/sessions/end?session_id={}", self.base_url, session_id);

        self.client
            .post(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<SessionResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn get_current_session(&self) -> Result<Option<SessionResponse>, String> {
        let url = format!("{}/sessions/current", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        // Check if response is null
        let text = response.text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if text == "null" {
            Ok(None)
        } else {
            serde_json::from_str(&text)
                .map_err(|e| format!("Failed to parse response: {}", e))
        }
    }

    pub fn get_all_sessions(&self) -> Result<Vec<SessionResponse>, String> {
        let url = format!("{}/sessions", self.base_url);

        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<Vec<SessionResponse>>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn get_session_statistics(&self) -> Result<Value, String> {
        let url = format!("{}/sessions/statistics", self.base_url);

        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<Value>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // Event Endpoints
    pub fn start_event(&self, slot_duration: Option<i32>, late_arrival_cutoff: Option<i32>, started_at: Option<String>) -> Result<EventSessionResponse, String> {
        let url = format!("{}/event/start", self.base_url);
        let request = StartEventRequest {
            slot_duration_minutes: slot_duration,
            late_arrival_cutoff_hours: late_arrival_cutoff,
            started_at,
        };

        self.client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<EventSessionResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn get_current_event(&self) -> Result<Option<EventSessionResponse>, String> {
        let url = format!("{}/event/current", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let text = response.text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if text == "null" || text == "No active event" {
            Ok(None)
        } else {
            serde_json::from_str(&text)
                .map_err(|e| format!("Failed to parse response: {}", e))
        }
    }

    pub fn end_event(&self) -> Result<EventSessionResponse, String> {
        let url = format!("{}/event/end", self.base_url);

        self.client
            .post(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?
            .json::<EventSessionResponse>()
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub fn get_timetable(&self) -> Result<Option<Timetable>, String> {
        let url = format!("{}/event/timetable", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        response.json::<Timetable>()
            .map(Some)
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}
