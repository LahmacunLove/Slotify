use crate::models::{
    dj::{Dj, DjResponse},
    lottery::{LotteryDraw, LotteryEngine, LotteryConfig, LotteryParticipant, LotteryStatistics},
    event_session::EventSession,
    AppState,
};
use anyhow::Result;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;

pub struct LotteryService {
    db: SqlitePool,
    engine: LotteryEngine,
}

impl LotteryService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        let config = LotteryConfig {
            base_weight: app_state.config.lottery_config.base_weight,
            late_arrival_penalty: app_state.config.lottery_config.late_arrival_penalty,
            time_block_hours: app_state.config.lottery_config.time_block_hours,
            enable_time_blocking: true,
        };

        Self {
            db: app_state.db.clone(),
            engine: LotteryEngine::new(config),
        }
    }

    pub async fn get_eligible_djs(&self) -> Result<Vec<Dj>> {
        let djs = sqlx::query_as::<_, Dj>(
            r#"
            SELECT id, name, email, registered_at, weight, is_active, position_in_queue
            FROM djs 
            WHERE is_active = true 
            AND position_in_queue IS NULL
            ORDER BY registered_at ASC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(djs)
    }

    pub async fn draw_next_dj(&self) -> Result<Option<LotteryDraw>> {
        let eligible_djs = self.get_eligible_djs().await?;

        if eligible_djs.is_empty() {
            return Ok(None);
        }

        // Get active event for late arrival penalty calculation
        let event = self.get_active_event().await?;

        let draw_result = self.engine.draw_winner(&eligible_djs, event.as_ref());

        if let Some(ref draw) = draw_result {
            // Save the draw to database
            self.save_lottery_draw(draw).await?;

            // Update the winner's position in queue
            self.assign_next_position(&draw.winner.id).await?;
        }

        Ok(draw_result)
    }

    async fn get_active_event(&self) -> Result<Option<EventSession>> {
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

    pub async fn save_lottery_draw(&self, draw: &LotteryDraw) -> Result<()> {
        let participants_json = serde_json::to_string(&draw.participants)?;
        
        sqlx::query(
            r#"
            INSERT INTO lottery_draws (id, winner_dj_id, drawn_at, algorithm_used, participants_data)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&uuid::Uuid::new_v4().to_string())
        .bind(&draw.winner.id)
        .bind(&draw.drawn_at)
        .bind(&draw.algorithm_used)
        .bind(&participants_json)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn assign_next_position(&self, dj_id: &str) -> Result<()> {
        // Get the highest position number currently assigned
        let max_position = sqlx::query("SELECT COALESCE(MAX(position_in_queue), 0) as max_pos FROM djs")
            .fetch_one(&self.db)
            .await?
            .get::<i32, _>("max_pos");

        // Assign the next position
        sqlx::query(
            "UPDATE djs SET position_in_queue = ? WHERE id = ?"
        )
        .bind(max_position + 1)
        .bind(dj_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_current_queue(&self) -> Result<Vec<DjResponse>> {
        let djs = sqlx::query_as::<_, Dj>(
            r#"
            SELECT id, name, email, registered_at, weight, is_active, position_in_queue
            FROM djs 
            WHERE position_in_queue IS NOT NULL 
            ORDER BY position_in_queue ASC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(djs.into_iter().map(|dj| dj.into()).collect())
    }

    pub async fn get_next_dj(&self) -> Result<Option<DjResponse>> {
        let dj = sqlx::query_as::<_, Dj>(
            r#"
            SELECT id, name, email, registered_at, weight, is_active, position_in_queue
            FROM djs 
            WHERE position_in_queue IS NOT NULL 
            ORDER BY position_in_queue ASC 
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(dj.map(|d| d.into()))
    }

    pub async fn remove_from_queue(&self, dj_id: &str) -> Result<()> {
        // Get the position of the DJ being removed
        let removed_position = sqlx::query(
            "SELECT position_in_queue FROM djs WHERE id = ?"
        )
        .bind(dj_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = removed_position {
            let position: Option<i32> = row.get("position_in_queue");
            
            // Remove the DJ from queue
            sqlx::query("UPDATE djs SET position_in_queue = NULL WHERE id = ?")
                .bind(dj_id)
                .execute(&self.db)
                .await?;

            // Shift everyone after this position down by 1
            if let Some(pos) = position {
                sqlx::query(
                    "UPDATE djs SET position_in_queue = position_in_queue - 1 WHERE position_in_queue > ?"
                )
                .bind(pos)
                .execute(&self.db)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn move_dj_position(&self, dj_id: &str, new_position: i32) -> Result<()> {
        // Get current position
        let current_row = sqlx::query(
            "SELECT position_in_queue FROM djs WHERE id = ?"
        )
        .bind(dj_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = current_row {
            let current_position: Option<i32> = row.get("position_in_queue");
            
            if let Some(current_pos) = current_position {
                // Temporarily set to -1 to avoid conflicts
                sqlx::query("UPDATE djs SET position_in_queue = -1 WHERE id = ?")
                    .bind(dj_id)
                    .execute(&self.db)
                    .await?;

                if new_position < current_pos {
                    // Moving up: shift others down
                    sqlx::query(
                        "UPDATE djs SET position_in_queue = position_in_queue + 1 
                         WHERE position_in_queue >= ? AND position_in_queue < ?"
                    )
                    .bind(new_position)
                    .bind(current_pos)
                    .execute(&self.db)
                    .await?;
                } else if new_position > current_pos {
                    // Moving down: shift others up
                    sqlx::query(
                        "UPDATE djs SET position_in_queue = position_in_queue - 1 
                         WHERE position_in_queue > ? AND position_in_queue <= ?"
                    )
                    .bind(current_pos)
                    .bind(new_position)
                    .execute(&self.db)
                    .await?;
                }

                // Set the new position
                sqlx::query("UPDATE djs SET position_in_queue = ? WHERE id = ?")
                    .bind(new_position)
                    .bind(dj_id)
                    .execute(&self.db)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn get_lottery_statistics(&self) -> Result<LotteryStatistics> {
        let total_draws = sqlx::query("SELECT COUNT(*) as count FROM lottery_draws")
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>("count") as usize;

        let unique_winners = sqlx::query(
            "SELECT COUNT(DISTINCT winner_dj_id) as count FROM lottery_draws"
        )
        .fetch_one(&self.db)
        .await?
        .get::<i64, _>("count") as usize;

        let average_weight = sqlx::query(
            "SELECT AVG(weight) as avg_weight FROM djs WHERE is_active = true"
        )
        .fetch_one(&self.db)
        .await?
        .get::<f64, _>("avg_weight");

        // Calculate fairness score (simplified: ratio of unique winners to total draws)
        let fairness_score = if total_draws > 0 {
            unique_winners as f64 / total_draws as f64
        } else {
            1.0
        };

        Ok(LotteryStatistics {
            total_draws,
            unique_winners,
            average_weight,
            fairness_score,
        })
    }

    pub async fn reset_lottery(&self) -> Result<()> {
        // Clear all queue positions
        sqlx::query("UPDATE djs SET position_in_queue = NULL")
            .execute(&self.db)
            .await?;

        Ok(())
    }
}