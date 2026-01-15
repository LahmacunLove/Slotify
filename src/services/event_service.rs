use crate::models::{
    event_session::{EventSession, EventSessionResponse, StartEventRequest, Timetable, TimetableEntry, TimetableEntryStatus},
    dj::Dj,
    session::Session,
    AppState,
};
use crate::services::LotteryService;
use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct EventService {
    db: SqlitePool,
    app_state: Arc<AppState>,
    default_slot_duration: i32,
    default_late_arrival_cutoff: i32,
}

impl EventService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            db: app_state.db.clone(),
            app_state: app_state.clone(),
            default_slot_duration: app_state.config.lottery_config.max_session_duration_minutes as i32,
            default_late_arrival_cutoff: app_state.config.lottery_config.time_block_hours as i32,
        }
    }

    pub async fn start_event(&self, request: StartEventRequest) -> Result<EventSessionResponse> {
        // Check if there's already an active event
        if let Some(_) = self.get_active_event().await? {
            return Err(anyhow!("An event is already running. End the current event first."));
        }

        let slot_duration = request.slot_duration_minutes.unwrap_or(self.default_slot_duration);
        let late_arrival_cutoff = request.late_arrival_cutoff_hours.unwrap_or(self.default_late_arrival_cutoff);

        let event = EventSession::new(slot_duration, late_arrival_cutoff, request.started_at);

        sqlx::query(
            r#"
            INSERT INTO event_sessions (id, started_at, ended_at, slot_duration_minutes,
                                       late_arrival_cutoff_hours, is_active, current_dj_id,
                                       current_slot_started_at, next_draw_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event.id)
        .bind(&event.started_at)
        .bind(&event.ended_at)
        .bind(&event.slot_duration_minutes)
        .bind(&event.late_arrival_cutoff_hours)
        .bind(&event.is_active)
        .bind(&event.current_dj_id)
        .bind(&event.current_slot_started_at)
        .bind(&event.next_draw_at)
        .execute(&self.db)
        .await?;

        // Automatically draw the first DJ
        let lottery_service = LotteryService::new(self.app_state.clone());
        if let Ok(Some(draw)) = lottery_service.draw_next_dj().await {
            tracing::info!("Automatically drew first DJ for new event: {}", draw.winner.name);
        } else {
            tracing::warn!("No DJs available to draw for the new event");
        }

        self.to_response(event).await
    }

    pub async fn get_active_event(&self) -> Result<Option<EventSession>> {
        let event = sqlx::query_as::<_, EventSession>(
            r#"
            SELECT * FROM event_sessions
            WHERE is_active = true AND ended_at IS NULL
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(event)
    }

    pub async fn get_active_event_response(&self) -> Result<Option<EventSessionResponse>> {
        if let Some(event) = self.get_active_event().await? {
            Ok(Some(self.to_response(event).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn end_event(&self) -> Result<EventSessionResponse> {
        let event = self.get_active_event().await?
            .ok_or_else(|| anyhow!("No active event found"))?;

        sqlx::query(
            r#"
            UPDATE event_sessions
            SET ended_at = ?, is_active = false
            WHERE id = ?
            "#,
        )
        .bind(Utc::now())
        .bind(&event.id)
        .execute(&self.db)
        .await?;

        let mut ended_event = event.clone();
        ended_event.ended_at = Some(Utc::now());
        ended_event.is_active = false;

        self.to_response(ended_event).await
    }

    pub async fn start_next_dj_slot(&self, dj_id: String) -> Result<EventSessionResponse> {
        let event = self.get_active_event().await?
            .ok_or_else(|| anyhow!("No active event found"))?;

        let slot_start = Utc::now();
        let next_draw = event.calculate_next_draw_time(slot_start);

        sqlx::query(
            r#"
            UPDATE event_sessions
            SET current_dj_id = ?, current_slot_started_at = ?, next_draw_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&dj_id)
        .bind(&slot_start)
        .bind(&next_draw)
        .bind(&event.id)
        .execute(&self.db)
        .await?;

        let mut updated_event = event;
        updated_event.current_dj_id = Some(dj_id);
        updated_event.current_slot_started_at = Some(slot_start);
        updated_event.next_draw_at = Some(next_draw);

        self.to_response(updated_event).await
    }

    pub async fn check_and_trigger_auto_draw(&self) -> Result<bool> {
        let event = match self.get_active_event().await? {
            Some(e) => e,
            None => return Ok(false),
        };

        if event.should_draw_next() {
            // Clear the next_draw_at so we don't trigger multiple times
            sqlx::query(
                r#"
                UPDATE event_sessions
                SET next_draw_at = NULL
                WHERE id = ?
                "#,
            )
            .bind(&event.id)
            .execute(&self.db)
            .await?;

            return Ok(true);
        }

        Ok(false)
    }

    pub async fn get_timetable(&self) -> Result<Option<Timetable>> {
        let event = match self.get_active_event().await? {
            Some(e) => e,
            None => return Ok(None),
        };

        // Get all queued DJs (ordered by position_in_queue)
        let queued_djs = sqlx::query_as::<_, Dj>(
            r#"
            SELECT * FROM djs
            WHERE position_in_queue IS NOT NULL
            ORDER BY position_in_queue ASC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        let mut entries = Vec::new();
        let mut completed_sets = 0;

        // Build timetable entries in queue order
        for (index, dj) in queued_djs.iter().enumerate() {
            let position = (index + 1) as i32;

            // Check if this DJ has a session
            let session = sqlx::query_as::<_, Session>(
                r#"
                SELECT * FROM sessions
                WHERE dj_id = ? AND started_at >= ?
                ORDER BY started_at DESC
                LIMIT 1
                "#,
            )
            .bind(&dj.id)
            .bind(&event.started_at)
            .fetch_optional(&self.db)
            .await?;

            let (started_at, ended_at, duration_minutes, status) = if let Some(session) = session {
                let status = if session.ended_at.is_some() {
                    completed_sets += 1;
                    TimetableEntryStatus::Completed
                } else if Some(dj.id.clone()) == event.current_dj_id {
                    TimetableEntryStatus::InProgress
                } else {
                    TimetableEntryStatus::Upcoming
                };
                (session.started_at, session.ended_at, session.duration_minutes, status)
            } else {
                // DJ is queued but hasn't started yet
                (event.started_at, None, None, TimetableEntryStatus::Upcoming)
            };

            entries.push(TimetableEntry {
                position,
                dj_id: dj.id.clone(),
                dj_name: dj.name.clone(),
                started_at,
                ended_at,
                duration_minutes,
                status,
            });
        }

        let total_djs = entries.len();

        Ok(Some(Timetable {
            event_id: event.id,
            event_started_at: event.started_at,
            entries,
            total_djs,
            completed_sets,
        }))
    }

    async fn to_response(&self, event: EventSession) -> Result<EventSessionResponse> {
        let current_dj_name = if let Some(ref dj_id) = event.current_dj_id {
            let dj = sqlx::query_as::<_, Dj>(
                "SELECT * FROM djs WHERE id = ?"
            )
            .bind(dj_id)
            .fetch_optional(&self.db)
            .await?;
            dj.map(|d| d.name)
        } else {
            None
        };

        // Calculate values before moving event fields
        let is_active = event.is_active();
        let elapsed_minutes = event.elapsed_minutes();
        let current_slot_progress_percent = event.current_slot_progress_percent();

        Ok(EventSessionResponse {
            id: event.id,
            started_at: event.started_at,
            ended_at: event.ended_at,
            slot_duration_minutes: event.slot_duration_minutes,
            late_arrival_cutoff_hours: event.late_arrival_cutoff_hours,
            is_active,
            current_dj_id: event.current_dj_id,
            current_dj_name,
            current_slot_started_at: event.current_slot_started_at,
            next_draw_at: event.next_draw_at,
            elapsed_minutes,
            current_slot_progress_percent,
        })
    }
}
