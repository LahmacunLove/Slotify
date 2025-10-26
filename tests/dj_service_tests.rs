use session_recorder_addon::{
    models::{
        AppState,
        dj::{CreateDjRequest, UpdateDjRequest},
    },
    services::DjService,
};
use sqlx::SqlitePool;
use std::sync::Arc;

#[cfg(test)]
mod dj_service_tests {
    use super::*;

    async fn setup_test_db() -> Arc<AppState> {
        // Create in-memory SQLite database for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        
        // Run migrations
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

    #[tokio::test]
    async fn test_register_dj() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        let request = CreateDjRequest {
            name: "Test DJ".to_string(),
            email: Some("test@example.com".to_string()),
        };
        
        let result = dj_service.register_dj(request).await;
        assert!(result.is_ok());
        
        let dj = result.unwrap();
        assert_eq!(dj.name, "Test DJ");
        assert_eq!(dj.email, Some("test@example.com".to_string()));
        assert!(dj.is_active);
        assert_eq!(dj.weight, 1.0);
    }

    #[tokio::test]
    async fn test_get_all_djs() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        // Register a few DJs
        let requests = vec![
            CreateDjRequest {
                name: "DJ One".to_string(),
                email: Some("one@example.com".to_string()),
            },
            CreateDjRequest {
                name: "DJ Two".to_string(),
                email: Some("two@example.com".to_string()),
            },
        ];
        
        for request in requests {
            dj_service.register_dj(request).await.unwrap();
        }
        
        let djs = dj_service.get_all_djs().await.unwrap();
        assert_eq!(djs.len(), 2);
        
        let names: Vec<&String> = djs.iter().map(|dj| &dj.name).collect();
        assert!(names.contains(&&"DJ One".to_string()));
        assert!(names.contains(&&"DJ Two".to_string()));
    }

    #[tokio::test]
    async fn test_get_active_djs_only() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        // Register and then deactivate one DJ
        let dj1 = dj_service.register_dj(CreateDjRequest {
            name: "Active DJ".to_string(),
            email: None,
        }).await.unwrap();
        
        let dj2 = dj_service.register_dj(CreateDjRequest {
            name: "Inactive DJ".to_string(),
            email: None,
        }).await.unwrap();
        
        // Deactivate second DJ
        dj_service.update_dj(&dj2.id, UpdateDjRequest {
            name: None,
            email: None,
            weight: None,
            is_active: Some(false),
            position_in_queue: None,
        }).await.unwrap();
        
        let active_djs = dj_service.get_active_djs().await.unwrap();
        assert_eq!(active_djs.len(), 1);
        assert_eq!(active_djs[0].name, "Active DJ");
    }

    #[tokio::test]
    async fn test_update_dj() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        let dj = dj_service.register_dj(CreateDjRequest {
            name: "Original Name".to_string(),
            email: Some("original@example.com".to_string()),
        }).await.unwrap();
        
        let update_request = UpdateDjRequest {
            name: Some("Updated Name".to_string()),
            email: Some("updated@example.com".to_string()),
            weight: Some(2.0),
            is_active: None,
            position_in_queue: None,
        };
        
        let updated_dj = dj_service.update_dj(&dj.id, update_request).await.unwrap();
        assert!(updated_dj.is_some());
        
        let updated_dj = updated_dj.unwrap();
        assert_eq!(updated_dj.name, "Updated Name");
        assert_eq!(updated_dj.email, Some("updated@example.com".to_string()));
        assert_eq!(updated_dj.weight, 2.0);
    }

    #[tokio::test]
    async fn test_remove_dj() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        let dj = dj_service.register_dj(CreateDjRequest {
            name: "To Be Removed".to_string(),
            email: None,
        }).await.unwrap();
        
        let removed = dj_service.remove_dj(&dj.id).await.unwrap();
        assert!(removed);
        
        let found_dj = dj_service.get_dj_by_id(&dj.id).await.unwrap();
        assert!(found_dj.is_none());
    }

    #[tokio::test]
    async fn test_get_dj_pool() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        // Register a few DJs
        for i in 1..=3 {
            dj_service.register_dj(CreateDjRequest {
                name: format!("DJ {}", i),
                email: Some(format!("dj{}@example.com", i)),
            }).await.unwrap();
        }
        
        let pool = dj_service.get_dj_pool().await.unwrap();
        assert_eq!(pool.active_djs.len(), 3);
        assert_eq!(pool.total_count, 3);
        assert!(pool.current_dj.is_none()); // No active sessions
        assert!(pool.next_dj.is_none()); // No queue positions assigned
    }

    #[tokio::test]
    async fn test_count_active_djs() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        // Initially no DJs
        let count = dj_service.count_active_djs().await.unwrap();
        assert_eq!(count, 0);
        
        // Register some DJs
        for i in 1..=5 {
            dj_service.register_dj(CreateDjRequest {
                name: format!("DJ {}", i),
                email: None,
            }).await.unwrap();
        }
        
        let count = dj_service.count_active_djs().await.unwrap();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_register_dj_without_email() {
        let app_state = setup_test_db().await;
        let dj_service = DjService::new(app_state);
        
        let request = CreateDjRequest {
            name: "No Email DJ".to_string(),
            email: None,
        };
        
        let result = dj_service.register_dj(request).await;
        assert!(result.is_ok());
        
        let dj = result.unwrap();
        assert_eq!(dj.name, "No Email DJ");
        assert!(dj.email.is_none());
    }
}