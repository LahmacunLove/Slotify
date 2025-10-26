use session_recorder_addon::{
    models::{
        AppState,
        dj::CreateDjRequest,
        session::{StartSessionRequest, SessionType},
    },
    services::{DjService, SessionService},
};
use sqlx::SqlitePool;
use std::sync::Arc;

#[cfg(test)]
mod session_tests {
    use super::*;

    async fn setup_test_db() -> Arc<AppState> {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        let config = session_recorder_addon::models::AppConfig {
            database_url: "sqlite::memory:".to_string(),
            cloud_storage_url: None,
            email_config: session_recorder_addon::models::EmailConfig {
                smtp_server: "localhost".to_string(),
                smtp_port: 587,
                username: "test".to_string(),
                password: "test".to_string(),
                from_address: "test@example.com".to_string(),
            },
            lottery_config: session_recorder_addon::models::LotteryConfig::default(),
        };
        
        Arc::new(AppState { db: pool, config })
    }

    async fn create_test_dj(app_state: &Arc<AppState>, name: &str) -> String {
        let dj_service = DjService::new(app_state.clone());
        let dj = dj_service.register_dj(CreateDjRequest {
            name: name.to_string(),
            email: Some(format!("{}@example.com", name.to_lowercase())),
        }).await.unwrap();
        dj.id
    }

    #[tokio::test]
    async fn test_start_session() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        let dj_id = create_test_dj(&app_state, "TestDJ").await;
        
        let request = StartSessionRequest {
            dj_id: dj_id.clone(),
            session_type: Some(SessionType::Solo),
        };
        
        let session = session_service.start_session(request).await.unwrap();
        
        assert_eq!(session.dj_id, dj_id);
        assert_eq!(session.dj_name, "TestDJ");
        assert!(session.ended_at.is_none());
        assert_eq!(session.session_type, SessionType::Solo);
    }

    #[tokio::test]
    async fn test_prevent_duplicate_sessions() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        let dj_id = create_test_dj(&app_state, "TestDJ").await;
        
        let request = StartSessionRequest {
            dj_id: dj_id.clone(),
            session_type: Some(SessionType::Solo),
        };
        
        // Start first session
        let session1 = session_service.start_session(request.clone()).await;
        assert!(session1.is_ok());
        
        // Try to start second session for same DJ
        let session2 = session_service.start_session(request).await;
        assert!(session2.is_err());
    }

    #[tokio::test]
    async fn test_end_session() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        let dj_id = create_test_dj(&app_state, "TestDJ").await;
        
        let request = StartSessionRequest {
            dj_id,
            session_type: Some(SessionType::Solo),
        };
        
        let session = session_service.start_session(request).await.unwrap();
        
        // Wait a bit to ensure duration is calculated
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let ended_session = session_service.end_session(&session.id).await.unwrap();
        assert!(ended_session.is_some());
        
        let ended_session = ended_session.unwrap();
        assert!(ended_session.ended_at.is_some());
        assert!(ended_session.duration_minutes.is_some());
        assert!(ended_session.duration_minutes.unwrap() >= 0);
    }

    #[tokio::test]
    async fn test_get_current_session() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        
        // Initially no current session
        let current = session_service.get_current_session().await.unwrap();
        assert!(current.is_none());
        
        // Start a session
        let dj_id = create_test_dj(&app_state, "CurrentDJ").await;
        let request = StartSessionRequest {
            dj_id,
            session_type: Some(SessionType::Solo),
        };
        let session = session_service.start_session(request).await.unwrap();
        
        // Now should have current session
        let current = session_service.get_current_session().await.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().id, session.id);
        
        // End session
        session_service.end_session(&session.id).await.unwrap();
        
        // Should not have current session anymore
        let current = session_service.get_current_session().await.unwrap();
        assert!(current.is_none());
    }

    #[tokio::test]
    async fn test_session_statistics() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        
        // Initial stats
        let stats = session_service.get_session_statistics().await.unwrap();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.active_sessions, 0);
        
        // Create some sessions
        let dj1_id = create_test_dj(&app_state, "DJ1").await;
        let dj2_id = create_test_dj(&app_state, "DJ2").await;
        
        let session1 = session_service.start_session(StartSessionRequest {
            dj_id: dj1_id,
            session_type: Some(SessionType::Solo),
        }).await.unwrap();
        
        let session2 = session_service.start_session(StartSessionRequest {
            dj_id: dj2_id,
            session_type: Some(SessionType::Solo),
        }).await.unwrap();
        
        // Check stats with active sessions
        let stats = session_service.get_session_statistics().await.unwrap();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.active_sessions, 2);
        
        // End one session
        session_service.end_session(&session1.id).await.unwrap();
        
        let stats = session_service.get_session_statistics().await.unwrap();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.active_sessions, 1);
    }

    #[tokio::test]
    async fn test_b2b_session() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        
        let dj1_id = create_test_dj(&app_state, "DJ1").await;
        let dj2_id = create_test_dj(&app_state, "DJ2").await;
        
        let b2b_request = session_recorder_addon::models::session::B2BSessionRequest {
            dj_ids: vec![dj1_id.clone(), dj2_id.clone()],
            duration_minutes: Some(120),
        };
        
        let session = session_service.create_b2b_session(b2b_request).await.unwrap();
        
        assert_eq!(session.session_type, SessionType::B2B);
        assert_eq!(session.dj_id, dj1_id); // Primary DJ
        assert!(session.dj_name.contains("+ 1 others")); // Indicates B2B
    }

    #[tokio::test]
    async fn test_b2b_session_requires_multiple_djs() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        
        let dj1_id = create_test_dj(&app_state, "DJ1").await;
        
        let b2b_request = session_recorder_addon::models::session::B2BSessionRequest {
            dj_ids: vec![dj1_id], // Only one DJ
            duration_minutes: Some(120),
        };
        
        let result = session_service.create_b2b_session(b2b_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_sessions() {
        let app_state = setup_test_db().await;
        let session_service = SessionService::new(app_state.clone());
        
        // Create multiple sessions
        let dj1_id = create_test_dj(&app_state, "DJ1").await;
        let dj2_id = create_test_dj(&app_state, "DJ2").await;
        
        session_service.start_session(StartSessionRequest {
            dj_id: dj1_id,
            session_type: Some(SessionType::Solo),
        }).await.unwrap();
        
        session_service.start_session(StartSessionRequest {
            dj_id: dj2_id,
            session_type: Some(SessionType::Solo),
        }).await.unwrap();
        
        let sessions = session_service.get_all_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);
        
        // Should be ordered by started_at DESC (most recent first)
        assert!(sessions[0].started_at >= sessions[1].started_at);
    }
}