use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite};
use std::collections::HashMap;
use uuid::Uuid;

pub mod dj;
pub mod session;
pub mod lottery;
pub mod event_session;

pub use dj::*;
pub use session::*;
pub use lottery::*;
pub use event_session::*;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: AppConfig,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub cloud_storage_url: Option<String>,
    pub email_config: EmailConfig,
    pub lottery_config: LotteryConfig,
    pub session_recorder_config: SessionRecorderIntegrationConfig,
}

#[derive(Clone, Debug)]
pub struct SessionRecorderIntegrationConfig {
    pub enabled: bool,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub bucket_name: String,
    pub public_endpoint: String,
    pub auto_link_tolerance_minutes: i64,
}

#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Clone, Debug)]
pub struct LotteryConfig {
    pub base_weight: f64,
    pub late_arrival_penalty: f64,
    pub time_block_hours: u32,
    pub max_session_duration_minutes: u32,
}

impl Default for LotteryConfig {
    fn default() -> Self {
        Self {
            base_weight: 1.0,
            late_arrival_penalty: 0.5,
            time_block_hours: 2,
            max_session_duration_minutes: 60,
        }
    }
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        let config = AppConfig::from_env()?;
        
        // Create database if it doesn't exist
        if !Sqlite::database_exists(&config.database_url).await.unwrap_or(false) {
            Sqlite::create_database(&config.database_url).await?;
        }

        let db = SqlitePool::connect(&config.database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&db).await?;

        Ok(Self { db, config })
    }
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:///tmp/dj_system.db".to_string()),
            cloud_storage_url: std::env::var("CLOUD_STORAGE_URL").ok(),
            email_config: EmailConfig {
                smtp_server: std::env::var("EMAIL_SMTP_SERVER")
                    .unwrap_or_else(|_| "localhost".to_string()),
                smtp_port: std::env::var("EMAIL_SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()?,
                username: std::env::var("EMAIL_USERNAME")
                    .unwrap_or_else(|_| "dj-system".to_string()),
                password: std::env::var("EMAIL_PASSWORD")
                    .unwrap_or_else(|_| "password".to_string()),
                from_address: std::env::var("EMAIL_FROM")
                    .unwrap_or_else(|_| "noreply@dj-system.local".to_string()),
            },
            lottery_config: LotteryConfig::default(),
            session_recorder_config: SessionRecorderIntegrationConfig {
                enabled: std::env::var("SESSION_RECORDER_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                minio_endpoint: std::env::var("SESSION_RECORDER_MINIO_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:9000".to_string()),
                minio_access_key: std::env::var("SESSION_RECORDER_MINIO_ACCESS_KEY")
                    .unwrap_or_else(|_| "admin".to_string()),
                minio_secret_key: std::env::var("SESSION_RECORDER_MINIO_SECRET_KEY")
                    .unwrap_or_else(|_| "password123".to_string()),
                bucket_name: std::env::var("SESSION_RECORDER_BUCKET_NAME")
                    .unwrap_or_else(|_| "session-recorder".to_string()),
                public_endpoint: std::env::var("SESSION_RECORDER_PUBLIC_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:9000".to_string()),
                auto_link_tolerance_minutes: std::env::var("SESSION_RECORDER_AUTO_LINK_TOLERANCE")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
        })
    }
}