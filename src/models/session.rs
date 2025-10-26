use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub dj_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub file_path: Option<String>,
    pub download_link: Option<String>,
    pub upload_status: SessionUploadStatus,
    pub session_type: SessionType,
    
    // Session-Recorder Integration
    pub recorder_session_id: Option<String>,
    pub recorder_id: Option<String>,
    pub recorder_ogg_url: Option<String>,
    pub recorder_flac_url: Option<String>,
    pub recorder_waveform_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "session_upload_status", rename_all = "lowercase")]
pub enum SessionUploadStatus {
    Recording,
    Processing,
    Uploaded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "session_type", rename_all = "lowercase")]
pub enum SessionType {
    Solo,
    B2B,
    Special,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartSessionRequest {
    pub dj_id: String,
    pub session_type: Option<SessionType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub dj_id: String,
    pub dj_name: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub download_link: Option<String>,
    pub upload_status: SessionUploadStatus,
    pub session_type: SessionType,
}

impl Session {
    pub fn new(dj_id: String, session_type: SessionType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            dj_id,
            started_at: Utc::now(),
            ended_at: None,
            duration_minutes: None,
            file_path: None,
            download_link: None,
            upload_status: SessionUploadStatus::Recording,
            session_type,
            recorder_session_id: None,
            recorder_id: None,
            recorder_ogg_url: None,
            recorder_flac_url: None,
            recorder_waveform_url: None,
        }
    }

    pub fn end_session(&mut self) -> anyhow::Result<()> {
        if self.ended_at.is_some() {
            return Err(anyhow::anyhow!("Session already ended"));
        }

        let now = Utc::now();
        self.ended_at = Some(now);
        self.duration_minutes = Some(
            now.signed_duration_since(self.started_at).num_minutes() as i32
        );
        self.upload_status = SessionUploadStatus::Processing;

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match self.ended_at {
            Some(end) => Some(end.signed_duration_since(self.started_at)),
            None => Some(Utc::now().signed_duration_since(self.started_at)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub average_duration_minutes: f64,
    pub total_duration_hours: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct B2BSessionRequest {
    pub dj_ids: Vec<String>,
    pub duration_minutes: Option<i32>,
}