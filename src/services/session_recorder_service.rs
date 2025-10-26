use anyhow::Result;
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::{Client as S3Client, Config as S3Config};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecorderSession {
    pub id: String,
    pub recorder_id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub is_closed: bool,
    pub keep: bool,
    pub files: SessionFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFiles {
    pub ogg_url: Option<String>,
    pub flac_url: Option<String>,
    pub waveform_url: Option<String>,
    pub overview_png_url: Option<String>,
    pub metadata_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub recorder_id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration: Option<i64>,
    pub keep: bool,
    pub is_closed: bool,
}

#[derive(Debug, Clone)]
pub struct SessionRecorderConfig {
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub bucket_name: String,
    pub public_endpoint: String,
}

impl Default for SessionRecorderConfig {
    fn default() -> Self {
        Self {
            minio_endpoint: "http://localhost:9000".to_string(),
            minio_access_key: "admin".to_string(),
            minio_secret_key: "password123".to_string(),
            bucket_name: "session-recorder".to_string(),
            public_endpoint: "http://localhost:9000".to_string(),
        }
    }
}

pub struct SessionRecorderService {
    s3_client: S3Client,
    config: SessionRecorderConfig,
}

impl SessionRecorderService {
    pub async fn new(config: SessionRecorderConfig) -> Result<Self> {
        let credentials = Credentials::new(
            &config.minio_access_key,
            &config.minio_secret_key,
            None,
            None,
            "session-recorder-integration",
        );

        let s3_config = S3Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .endpoint_url(&config.minio_endpoint)
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let s3_client = S3Client::from_conf(s3_config);

        Ok(Self { s3_client, config })
    }

    /// Get all available recorders from MinIO
    pub async fn get_recorders(&self) -> Result<Vec<String>> {
        let response = self
            .s3_client
            .list_objects_v2()
            .bucket(&self.config.bucket_name)
            .delimiter("/")
            .send()
            .await?;

        let mut recorders = Vec::new();
        
        for prefix in response.common_prefixes() {
            if let Some(prefix_str) = prefix.prefix() {
                // Remove trailing slash and add to recorders
                let recorder_id = prefix_str.trim_end_matches('/');
                // Skip metadata.json at root level
                if recorder_id != "metadata.json" && !recorder_id.is_empty() {
                    recorders.push(recorder_id.to_string());
                }
            }
        }

        Ok(recorders)
    }

    /// Get all sessions for a specific recorder
    pub async fn get_sessions(&self, recorder_id: &str) -> Result<Vec<RecorderSession>> {
        let sessions_prefix = format!("{}/sessions/", recorder_id);
        
        let response = self
            .s3_client
            .list_objects_v2()
            .bucket(&self.config.bucket_name)
            .prefix(&sessions_prefix)
            .delimiter("/")
            .send()
            .await?;

        let mut sessions = Vec::new();

        for prefix in response.common_prefixes() {
            if let Some(prefix_str) = prefix.prefix() {
                // Extract session ID from path like "recorder-id/sessions/session-id/"
                let session_path = prefix_str.trim_end_matches('/');
                if let Some(session_id) = session_path.split('/').last() {
                    if let Ok(session) = self.get_session_details(recorder_id, session_id).await {
                        sessions.push(session);
                    }
                }
            }
        }

        Ok(sessions)
    }

    /// Get detailed information about a specific session
    pub async fn get_session_details(&self, recorder_id: &str, session_id: &str) -> Result<RecorderSession> {
        let metadata_key = format!("{}/sessions/{}/metadata.json", recorder_id, session_id);
        
        // Try to get metadata
        let metadata = match self.get_session_metadata(&metadata_key).await {
            Ok(meta) => meta,
            Err(_) => {
                // If metadata doesn't exist, create a basic session info
                SessionMetadata {
                    id: session_id.to_string(),
                    recorder_id: recorder_id.to_string(),
                    name: format!("Session {}", session_id),
                    start_time: Utc::now().to_rfc3339(),
                    end_time: None,
                    duration: None,
                    keep: false,
                    is_closed: true,
                }
            }
        };

        // Generate file URLs
        let files = self.get_session_files(recorder_id, session_id).await?;

        let start_time = DateTime::parse_from_rfc3339(&metadata.start_time)
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);

        let end_time = metadata.end_time
            .as_ref()
            .and_then(|et| DateTime::parse_from_rfc3339(et).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(RecorderSession {
            id: metadata.id,
            recorder_id: metadata.recorder_id,
            name: metadata.name,
            start_time,
            end_time,
            duration_seconds: metadata.duration,
            is_closed: metadata.is_closed,
            keep: metadata.keep,
            files,
        })
    }

    /// Get session metadata from MinIO
    async fn get_session_metadata(&self, metadata_key: &str) -> Result<SessionMetadata> {
        let response = self
            .s3_client
            .get_object()
            .bucket(&self.config.bucket_name)
            .key(metadata_key)
            .send()
            .await?;

        let body = response.body.collect().await?.into_bytes();
        let metadata: SessionMetadata = serde_json::from_slice(&body)?;
        
        Ok(metadata)
    }

    /// Get available file URLs for a session
    async fn get_session_files(&self, recorder_id: &str, session_id: &str) -> Result<SessionFiles> {
        let session_prefix = format!("{}/sessions/{}/", recorder_id, session_id);
        
        let response = self
            .s3_client
            .list_objects_v2()
            .bucket(&self.config.bucket_name)
            .prefix(&session_prefix)
            .send()
            .await?;

        let mut files = SessionFiles {
            ogg_url: None,
            flac_url: None,
            waveform_url: None,
            overview_png_url: None,
            metadata_url: None,
        };

        for object in response.contents() {
            if let Some(key) = object.key() {
                if let Some(filename) = key.split('/').last() {
                    let url = format!("{}/{}", self.config.public_endpoint, key);
                    
                    match filename {
                        "data.ogg" => files.ogg_url = Some(url),
                        "data.flac" => files.flac_url = Some(url),
                        "waveform.dat" => files.waveform_url = Some(url),
                        "overview.png" => files.overview_png_url = Some(url),
                        "metadata.json" => files.metadata_url = Some(url),
                        _ => {}
                    }
                }
            }
        }

        Ok(files)
    }

    /// Download a session file (OGG, FLAC, etc.)
    pub async fn download_session_file(&self, recorder_id: &str, session_id: &str, filename: &str) -> Result<Vec<u8>> {
        let key = format!("{}/sessions/{}/{}", recorder_id, session_id, filename);
        
        let response = self
            .s3_client
            .get_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .send()
            .await?;

        let body = response.body.collect().await?.into_bytes();
        Ok(body.to_vec())
    }

    /// Get presigned URL for direct access to a file
    pub async fn get_presigned_url(&self, recorder_id: &str, session_id: &str, filename: &str, expires_in_secs: u64) -> Result<String> {
        let key = format!("{}/sessions/{}/{}", recorder_id, session_id, filename);
        
        let presigned_request = self
            .s3_client
            .get_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(expires_in_secs)
                ).unwrap()
            )
            .await?;

        Ok(presigned_request.uri().to_string())
    }

    /// Map a DJ session to a recorder session based on timing
    pub async fn find_matching_session(&self, dj_session_start: DateTime<Utc>, tolerance_minutes: i64) -> Result<Option<RecorderSession>> {
        let recorders = self.get_recorders().await?;
        
        for recorder_id in recorders {
            let sessions = self.get_sessions(&recorder_id).await?;
            
            for session in sessions {
                // Check if the session start time is within tolerance
                let time_diff = (session.start_time - dj_session_start).num_minutes().abs();
                
                if time_diff <= tolerance_minutes {
                    return Ok(Some(session));
                }
            }
        }
        
        Ok(None)
    }

    /// Get all recent sessions (useful for automatic mapping)
    pub async fn get_recent_sessions(&self, hours: i64) -> Result<Vec<RecorderSession>> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours);
        let recorders = self.get_recorders().await?;
        
        let mut recent_sessions = Vec::new();
        
        for recorder_id in recorders {
            let sessions = self.get_sessions(&recorder_id).await?;
            
            for session in sessions {
                if session.start_time >= cutoff_time {
                    recent_sessions.push(session);
                }
            }
        }
        
        // Sort by start time (most recent first)
        recent_sessions.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        
        Ok(recent_sessions)
    }
}