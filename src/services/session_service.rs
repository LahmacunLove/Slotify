use crate::models::{
    session::{Session, SessionResponse, StartSessionRequest, SessionStats, SessionType, SessionUploadStatus, B2BSessionRequest},
    AppState,
};
use crate::services::{SessionRecorderService, SessionRecorderConfig, EventService};
use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct SessionService {
    db: SqlitePool,
    app_state: Arc<AppState>,
    session_recorder: Option<SessionRecorderService>,
}

impl SessionService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            db: app_state.db.clone(),
            app_state: app_state.clone(),
            session_recorder: None,
        }
    }

    pub async fn new_with_recorder(app_state: Arc<AppState>) -> Result<Self> {
        let recorder_config = SessionRecorderConfig::default(); // TODO: Load from config
        let session_recorder = SessionRecorderService::new(recorder_config).await.ok();

        Ok(Self {
            db: app_state.db.clone(),
            app_state: app_state.clone(),
            session_recorder,
        })
    }

    pub async fn start_session(&self, request: StartSessionRequest) -> Result<SessionResponse> {
        // Check if DJ exists and is not already in an active session
        let existing_session = sqlx::query(
            "SELECT id FROM sessions WHERE dj_id = ? AND ended_at IS NULL"
        )
        .bind(&request.dj_id)
        .fetch_optional(&self.db)
        .await?;

        if existing_session.is_some() {
            return Err(anyhow::anyhow!("DJ already has an active session"));
        }

        let session_type = request.session_type.unwrap_or(SessionType::Solo);
        let session = Session::new(request.dj_id, session_type);

        sqlx::query(
            r#"
            INSERT INTO sessions (id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, upload_status, session_type)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.dj_id)
        .bind(&session.started_at)
        .bind(&session.ended_at)
        .bind(&session.duration_minutes)
        .bind(&session.file_path)
        .bind(&session.download_link)
        .bind(&session.upload_status)
        .bind(&session.session_type)
        .execute(&self.db)
        .await?;

        // Update event session to track current DJ and set next draw time
        let event_service = EventService::new(self.app_state.clone());
        if let Err(e) = event_service.start_next_dj_slot(session.dj_id.clone()).await {
            tracing::warn!("Failed to update event session: {}", e);
        }

        // Get DJ name for response
        let dj_name = self.get_dj_name(&session.dj_id).await?;

        Ok(SessionResponse {
            id: session.id,
            dj_id: session.dj_id,
            dj_name,
            started_at: session.started_at,
            ended_at: session.ended_at,
            duration_minutes: session.duration_minutes,
            download_link: session.download_link,
            upload_status: session.upload_status,
            session_type: session.session_type,
        })
    }

    pub async fn end_session(&self, session_id: &str) -> Result<Option<SessionResponse>> {
        // Get the session
        let mut session = sqlx::query_as::<_, Session>(
            "SELECT id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, upload_status, session_type FROM sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(ref mut session) = session {
            session.end_session()?;

            // Update the session in database
            sqlx::query(
                "UPDATE sessions SET ended_at = ?, duration_minutes = ?, upload_status = ? WHERE id = ?"
            )
            .bind(&session.ended_at)
            .bind(&session.duration_minutes)
            .bind(&session.upload_status)
            .bind(session_id)
            .execute(&self.db)
            .await?;

            // Start file processing and cloud upload
            self.process_session_recording(session).await?;

            // Get DJ name for response
            let dj_name = self.get_dj_name(&session.dj_id).await?;

            Ok(Some(SessionResponse {
                id: session.id.clone(),
                dj_id: session.dj_id.clone(),
                dj_name,
                started_at: session.started_at,
                ended_at: session.ended_at,
                duration_minutes: session.duration_minutes,
                download_link: session.download_link.clone(),
                upload_status: session.upload_status.clone(),
                session_type: session.session_type.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_session_by_id(&self, id: &str) -> Result<Option<SessionResponse>> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, upload_status, session_type FROM sessions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(session) = session {
            let dj_name = self.get_dj_name(&session.dj_id).await?;
            
            Ok(Some(SessionResponse {
                id: session.id,
                dj_id: session.dj_id,
                dj_name,
                started_at: session.started_at,
                ended_at: session.ended_at,
                duration_minutes: session.duration_minutes,
                download_link: session.download_link,
                upload_status: session.upload_status,
                session_type: session.session_type,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_sessions(&self) -> Result<Vec<SessionResponse>> {
        let sessions = sqlx::query_as::<_, Session>(
            "SELECT id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, upload_status, session_type FROM sessions ORDER BY started_at DESC"
        )
        .fetch_all(&self.db)
        .await?;

        let mut responses = Vec::new();
        for session in sessions {
            let dj_name = self.get_dj_name(&session.dj_id).await?;
            responses.push(SessionResponse {
                id: session.id,
                dj_id: session.dj_id,
                dj_name,
                started_at: session.started_at,
                ended_at: session.ended_at,
                duration_minutes: session.duration_minutes,
                download_link: session.download_link,
                upload_status: session.upload_status,
                session_type: session.session_type,
            });
        }

        Ok(responses)
    }

    pub async fn get_current_session(&self) -> Result<Option<SessionResponse>> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, upload_status, session_type FROM sessions WHERE ended_at IS NULL ORDER BY started_at DESC LIMIT 1"
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(session) = session {
            let dj_name = self.get_dj_name(&session.dj_id).await?;
            
            Ok(Some(SessionResponse {
                id: session.id,
                dj_id: session.dj_id,
                dj_name,
                started_at: session.started_at,
                ended_at: session.ended_at,
                duration_minutes: session.duration_minutes,
                download_link: session.download_link,
                upload_status: session.upload_status,
                session_type: session.session_type,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_download_link(&self, session_id: &str) -> Result<Option<String>> {
        let link = sqlx::query(
            "SELECT download_link FROM sessions WHERE id = ? AND upload_status = 'uploaded'"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(link.and_then(|row| row.get("download_link")))
    }

    pub async fn get_session_statistics(&self) -> Result<SessionStats> {
        let total_sessions = sqlx::query("SELECT COUNT(*) as count FROM sessions")
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>("count") as usize;

        let active_sessions = sqlx::query("SELECT COUNT(*) as count FROM sessions WHERE ended_at IS NULL")
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>("count") as usize;

        let avg_duration = sqlx::query("SELECT AVG(duration_minutes) as avg_duration FROM sessions WHERE duration_minutes IS NOT NULL")
            .fetch_one(&self.db)
            .await?
            .get::<Option<f64>, _>("avg_duration")
            .unwrap_or(0.0);

        let total_duration = sqlx::query("SELECT SUM(duration_minutes) as total_duration FROM sessions WHERE duration_minutes IS NOT NULL")
            .fetch_one(&self.db)
            .await?
            .get::<Option<i64>, _>("total_duration")
            .unwrap_or(0) as f64 / 60.0; // Convert to hours

        Ok(SessionStats {
            total_sessions,
            active_sessions,
            average_duration_minutes: avg_duration,
            total_duration_hours: total_duration,
        })
    }

    pub async fn create_b2b_session(&self, request: B2BSessionRequest) -> Result<SessionResponse> {
        if request.dj_ids.len() < 2 {
            return Err(anyhow::anyhow!("B2B session requires at least 2 DJs"));
        }

        // For simplicity, create a session with the first DJ as primary
        let primary_dj_id = &request.dj_ids[0];
        let session = Session::new(primary_dj_id.clone(), SessionType::B2B);

        sqlx::query(
            r#"
            INSERT INTO sessions (id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, upload_status, session_type)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.dj_id)
        .bind(&session.started_at)
        .bind(&session.ended_at)
        .bind(&session.duration_minutes)
        .bind(&session.file_path)
        .bind(&session.download_link)
        .bind(&session.upload_status)
        .bind(&session.session_type)
        .execute(&self.db)
        .await?;

        // TODO: Store additional B2B participants in a separate table
        // For now, we'll just use the primary DJ

        let dj_name = self.get_dj_name(&session.dj_id).await?;

        Ok(SessionResponse {
            id: session.id,
            dj_id: session.dj_id,
            dj_name: format!("{} + {} others", dj_name, request.dj_ids.len() - 1),
            started_at: session.started_at,
            ended_at: session.ended_at,
            duration_minutes: session.duration_minutes,
            download_link: session.download_link,
            upload_status: session.upload_status,
            session_type: session.session_type,
        })
    }

    async fn get_dj_name(&self, dj_id: &str) -> Result<String> {
        let name = sqlx::query("SELECT name FROM djs WHERE id = ?")
            .bind(dj_id)
            .fetch_optional(&self.db)
            .await?
            .map(|row| row.get::<String, _>("name"))
            .unwrap_or_else(|| "Unknown DJ".to_string());

        Ok(name)
    }

    async fn process_session_recording(&self, session: &Session) -> Result<()> {
        // This would be where we:
        // 1. Process the audio recording
        // 2. Upload to cloud storage
        // 3. Generate download links
        // 4. Send email to DJ with download link

        // For now, simulate the process
        let session_id = session.id.clone();
        let download_link = format!("https://cloud-storage.example.com/sessions/{}.mp3", session_id);

        // Update session with download link
        sqlx::query(
            "UPDATE sessions SET download_link = ?, upload_status = 'uploaded' WHERE id = ?"
        )
        .bind(&download_link)
        .bind(&session_id)
        .execute(&self.db)
        .await?;

        // TODO: Send email to DJ with download link
        tracing::info!("Session {} processed and uploaded: {}", session_id, download_link);

        Ok(())
    }

    /// Link a DJ session to a session-recorder session
    pub async fn link_to_recorder_session(&self, session_id: &str, recorder_session_id: &str, recorder_id: &str) -> Result<()> {
        if let Some(ref recorder) = self.session_recorder {
            // Get session details from recorder
            if let Ok(recorder_session) = recorder.get_session_details(recorder_id, recorder_session_id).await {
                sqlx::query(
                    r#"
                    UPDATE sessions 
                    SET recorder_session_id = ?, recorder_id = ?, 
                        recorder_ogg_url = ?, recorder_flac_url = ?, recorder_waveform_url = ?
                    WHERE id = ?
                    "#,
                )
                .bind(recorder_session_id)
                .bind(recorder_id)
                .bind(&recorder_session.files.ogg_url)
                .bind(&recorder_session.files.flac_url)
                .bind(&recorder_session.files.waveform_url)
                .bind(session_id)
                .execute(&self.db)
                .await?;
            }
        }
        Ok(())
    }

    /// Try to automatically find and link a matching recorder session
    pub async fn auto_link_recorder_session(&self, session_id: &str, session_start_time: chrono::DateTime<chrono::Utc>) -> Result<bool> {
        if let Some(ref recorder) = self.session_recorder {
            // Look for sessions within 5 minutes of the DJ session start
            if let Ok(Some(recorder_session)) = recorder.find_matching_session(session_start_time, 5).await {
                self.link_to_recorder_session(session_id, &recorder_session.id, &recorder_session.recorder_id).await?;
                tracing::info!("Auto-linked DJ session {} to recorder session {}/{}", 
                              session_id, recorder_session.recorder_id, recorder_session.id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get all available recorder sessions
    pub async fn get_available_recorder_sessions(&self) -> Result<Vec<crate::services::RecorderSession>> {
        if let Some(ref recorder) = self.session_recorder {
            let recent_sessions = recorder.get_recent_sessions(24).await?;
            return Ok(recent_sessions);
        }
        Ok(Vec::new())
    }

    /// Get recording download URL for a session
    pub async fn get_recording_download_url(&self, session_id: &str, format: &str) -> Result<Option<String>> {
        let session = sqlx::query(
            "SELECT recorder_session_id, recorder_id FROM sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = session {
            let recorder_session_id: Option<String> = row.get("recorder_session_id");
            let recorder_id: Option<String> = row.get("recorder_id");

            if let (Some(rec_session_id), Some(rec_id), Some(ref recorder)) = 
                (recorder_session_id, recorder_id, &self.session_recorder) {
                
                let filename = match format.to_lowercase().as_str() {
                    "ogg" => "data.ogg",
                    "flac" => "data.flac",
                    "waveform" => "waveform.dat",
                    _ => return Ok(None),
                };

                let url = recorder.get_presigned_url(&rec_id, &rec_session_id, filename, 3600).await?;
                return Ok(Some(url));
            }
        }

        Ok(None)
    }

    /// Get session with recorder information
    pub async fn get_session_with_recorder_info(&self, session_id: &str) -> Result<Option<SessionResponse>> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, dj_id, started_at, ended_at, duration_minutes, file_path, download_link, 
                   upload_status, session_type, recorder_session_id, recorder_id, 
                   recorder_ogg_url, recorder_flac_url, recorder_waveform_url
            FROM sessions WHERE id = ?
            "#
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(session) = session {
            let dj_name = self.get_dj_name(&session.dj_id).await?;
            
            let mut response = SessionResponse {
                id: session.id,
                dj_id: session.dj_id,
                dj_name,
                started_at: session.started_at,
                ended_at: session.ended_at,
                duration_minutes: session.duration_minutes,
                download_link: session.download_link,
                upload_status: session.upload_status,
                session_type: session.session_type,
            };

            // If linked to recorder, prefer recorder URLs
            if session.recorder_ogg_url.is_some() {
                response.download_link = session.recorder_ogg_url;
            }

            Ok(Some(response))
        } else {
            Ok(None)
        }
    }
}