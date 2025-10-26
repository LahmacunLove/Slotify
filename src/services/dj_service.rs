use crate::models::{
    dj::{Dj, DjResponse, CreateDjRequest, UpdateDjRequest, DjPool, GuestRequest},
    AppState,
};
use anyhow::Result;
use sqlx::{SqlitePool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct DjService {
    db: SqlitePool,
}

impl DjService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            db: app_state.db.clone(),
        }
    }

    pub async fn register_dj(&self, request: CreateDjRequest) -> Result<DjResponse> {
        let dj = Dj::new(request.name, request.email);
        
        sqlx::query(
            r#"
            INSERT INTO djs (id, name, email, registered_at, weight, is_active, position_in_queue)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&dj.id)
        .bind(&dj.name)
        .bind(&dj.email)
        .bind(&dj.registered_at)
        .bind(dj.weight)
        .bind(dj.is_active)
        .bind(dj.position_in_queue)
        .execute(&self.db)
        .await?;

        Ok(dj.into())
    }

    pub async fn get_all_djs(&self) -> Result<Vec<DjResponse>> {
        let djs = sqlx::query_as::<_, Dj>(
            "SELECT id, name, email, registered_at, weight, is_active, position_in_queue FROM djs ORDER BY registered_at ASC"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(djs.into_iter().map(|dj| dj.into()).collect())
    }

    pub async fn get_active_djs(&self) -> Result<Vec<DjResponse>> {
        let djs = sqlx::query_as::<_, Dj>(
            "SELECT id, name, email, registered_at, weight, is_active, position_in_queue FROM djs WHERE is_active = true ORDER BY registered_at ASC"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(djs.into_iter().map(|dj| dj.into()).collect())
    }

    pub async fn get_dj_by_id(&self, id: &str) -> Result<Option<DjResponse>> {
        let dj = sqlx::query_as::<_, Dj>(
            "SELECT id, name, email, registered_at, weight, is_active, position_in_queue FROM djs WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(dj.map(|d| d.into()))
    }

    pub async fn update_dj(&self, id: &str, request: UpdateDjRequest) -> Result<Option<DjResponse>> {
        // Simple approach: build query dynamically
        let mut _updates: Vec<String> = Vec::new();
        
        if request.name.is_some() || request.email.is_some() || request.weight.is_some() || 
           request.is_active.is_some() || request.position_in_queue.is_some() {
            
            if let Some(name) = &request.name {
                sqlx::query("UPDATE djs SET name = ? WHERE id = ?")
                    .bind(name)
                    .bind(id)
                    .execute(&self.db)
                    .await?;
            }
            
            if let Some(email) = &request.email {
                sqlx::query("UPDATE djs SET email = ? WHERE id = ?")
                    .bind(email)
                    .bind(id)
                    .execute(&self.db)
                    .await?;
            }
            
            if let Some(weight) = request.weight {
                sqlx::query("UPDATE djs SET weight = ? WHERE id = ?")
                    .bind(weight)
                    .bind(id)
                    .execute(&self.db)
                    .await?;
            }
            
            if let Some(is_active) = request.is_active {
                sqlx::query("UPDATE djs SET is_active = ? WHERE id = ?")
                    .bind(is_active)
                    .bind(id)
                    .execute(&self.db)
                    .await?;
            }
            
            if let Some(position) = request.position_in_queue {
                sqlx::query("UPDATE djs SET position_in_queue = ? WHERE id = ?")
                    .bind(position)
                    .bind(id)
                    .execute(&self.db)
                    .await?;
            }
        }

        self.get_dj_by_id(id).await
    }

    pub async fn remove_dj(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM djs WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_dj_pool(&self) -> Result<DjPool> {
        let active_djs = self.get_active_djs().await?;
        
        let current_dj = sqlx::query_as::<_, Dj>(
            r#"
            SELECT d.id, d.name, d.email, d.registered_at, d.weight, d.is_active, d.position_in_queue
            FROM djs d
            INNER JOIN sessions s ON d.id = s.dj_id
            WHERE s.ended_at IS NULL
            ORDER BY s.started_at DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.db)
        .await?;

        let next_dj = sqlx::query_as::<_, Dj>(
            "SELECT id, name, email, registered_at, weight, is_active, position_in_queue FROM djs WHERE position_in_queue = 1"
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(DjPool {
            active_djs,
            current_dj: current_dj.map(|d| d.into()),
            next_dj: next_dj.map(|d| d.into()),
            total_count: self.count_active_djs().await?,
        })
    }

    pub async fn count_active_djs(&self) -> Result<usize> {
        let count = sqlx::query("SELECT COUNT(*) as count FROM djs WHERE is_active = true")
            .fetch_one(&self.db)
            .await?
            .get::<i64, _>("count") as usize;

        Ok(count)
    }

    pub async fn submit_guest_request(&self, dj_id: String, request: GuestRequest) -> Result<()> {
        let request_id = Uuid::new_v4().to_string();
        
        sqlx::query(
            r#"
            INSERT INTO guest_requests (id, guest_name, guest_email, message, target_dj_id, status)
            VALUES (?, ?, ?, ?, ?, 'pending')
            "#,
        )
        .bind(&request_id)
        .bind(&request.guest_name)
        .bind(&request.guest_email)
        .bind(&request.message)
        .bind(&dj_id)
        .execute(&self.db)
        .await?;

        // TODO: Send email notification to DJ
        self.send_guest_request_email(&dj_id, &request).await?;

        Ok(())
    }

    async fn send_guest_request_email(&self, dj_id: &str, request: &GuestRequest) -> Result<()> {
        // TODO: Implement email sending using lettre
        // For now, just log the request
        tracing::info!(
            "Guest request from {} ({}) to DJ {} with message: {:?}",
            request.guest_name,
            request.guest_email,
            dj_id,
            request.message
        );

        Ok(())
    }

    pub async fn get_guest_requests(&self, dj_id: &str) -> Result<Vec<GuestRequest>> {
        let requests = sqlx::query_as::<_, GuestRequest>(
            "SELECT guest_name, guest_email, message, target_dj_id FROM guest_requests WHERE target_dj_id = ? AND status = 'pending' ORDER BY created_at ASC"
        )
        .bind(dj_id)
        .fetch_all(&self.db)
        .await?;

        Ok(requests)
    }

    pub async fn approve_guest_request(&self, request_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE guest_requests SET status = 'approved' WHERE id = ?"
        )
        .bind(request_id)
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn reject_guest_request(&self, request_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE guest_requests SET status = 'rejected' WHERE id = ?"
        )
        .bind(request_id)
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}