#[cfg(test)]
mod queue_tests {
    use trackex_agent_lib::storage::{database, offline_queue};
    use serde_json::json;
    use tempfile::tempdir;
    
    async fn setup_test_db() -> Result<(), Box<dyn std::error::Error>> {
        database::init().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_queue_event() {
        setup_test_db().await.unwrap();
        
        let event_data = json!({
            "app_name": "Test App",
            "timestamp": "2024-01-01T00:00:00Z"
        });
        
        let result = offline_queue::queue_event("app_focus", &event_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_pending_events() {
        setup_test_db().await.unwrap();
        
        // Queue some test events
        let event_data = json!({"test": "data"});
        offline_queue::queue_event("test_event", &event_data).await.unwrap();
        
        let events = offline_queue::get_pending_events().await.unwrap();
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, "test_event");
    }

    #[tokio::test]
    async fn test_mark_event_processed() {
        setup_test_db().await.unwrap();
        
        // Queue an event
        let event_data = json!({"test": "data"});
        offline_queue::queue_event("test_event", &event_data).await.unwrap();
        
        // Get the event
        let events = offline_queue::get_pending_events().await.unwrap();
        assert!(!events.is_empty());
        
        let event_id = events[0].id;
        
        // Mark as processed
        offline_queue::mark_event_processed(event_id).await.unwrap();
        
        // Should not appear in pending events anymore
        let pending_events = offline_queue::get_pending_events().await.unwrap();
        assert!(!pending_events.iter().any(|e| e.id == event_id));
    }

    #[tokio::test]
    async fn test_queue_heartbeat() {
        setup_test_db().await.unwrap();
        
        let heartbeat_data = json!({
            "status": "active",
            "timestamp": "2024-01-01T00:00:00Z"
        });
        
        let result = offline_queue::queue_heartbeat(&heartbeat_data).await;
        assert!(result.is_ok());
        
        let heartbeats = offline_queue::get_pending_heartbeats().await.unwrap();
        assert!(!heartbeats.is_empty());
    }

    #[tokio::test]
    async fn test_queue_stats() {
        setup_test_db().await.unwrap();
        
        // Queue some test data
        let event_data = json!({"test": "data"});
        offline_queue::queue_event("test_event", &event_data).await.unwrap();
        
        let heartbeat_data = json!({"status": "active"});
        offline_queue::queue_heartbeat(&heartbeat_data).await.unwrap();
        
        let (pending_events, _processed_events, pending_heartbeats, _processed_heartbeats) = 
            offline_queue::get_queue_stats().await.unwrap();
        
        assert!(pending_events > 0);
        assert!(pending_heartbeats > 0);
    }
}
