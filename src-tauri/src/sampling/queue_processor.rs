use tauri::AppHandle;
use tokio::time::Duration;

use crate::storage::offline_queue;

/// Start the offline queue processor
/// This service runs continuously to send queued events and heartbeats
/// It stops immediately after clock out to prevent data corruption
#[allow(dead_code)]
pub async fn start_queue_processor(_app_handle: AppHandle) {
    let processing_interval = Duration::from_secs(5); // Process queue every 5 seconds
    let mut interval = tokio::time::interval(processing_interval);
    
    log::info!("📦 Queue processor starting (interval: {}s)", processing_interval.as_secs());
    
    loop {
        interval.tick().await;
        
        // Check if we should continue running
        let is_clocked_in = super::should_services_run().await;
        
        if !is_clocked_in {
            // Not clocked in - stop immediately to prevent data corruption
            log::info!("Queue processor stopping - user clocked out or logged out");
            break;
        }
        
        // Process pending events
        match process_pending_events().await {
            Ok(count) => {
                if count > 0 {
                    log::info!("✓ Processed {} pending events", count);
                }
            }
            Err(e) => {
                log::error!("Failed to process pending events: {}", e);
            }
        }
        
        // Process pending heartbeats
        match process_pending_heartbeats().await {
            Ok(count) => {
                if count > 0 {
                    log::info!("✓ Processed {} pending heartbeats", count);
                }
            }
            Err(e) => {
                log::error!("Failed to process pending heartbeats: {}", e);
            }
        }
    }
    
    log::info!("Queue processor stopped");
}

async fn process_pending_events() -> anyhow::Result<usize> {
    let pending_events = offline_queue::get_pending_events().await?;
    let count = pending_events.len();
    
    for event in pending_events {
        // Try to send the event
        match super::send_event_to_backend(&event.event_type, &event.event_data).await {
            Ok(_) => {
                // Mark as processed
                offline_queue::mark_event_processed(event.id).await?;
                log::debug!("✓ Sent queued {} event", event.event_type);
            }
            Err(e) => {
                // Mark as failed (increment retry count)
                offline_queue::mark_event_failed(event.id).await?;
                log::warn!("Failed to send queued {} event (retry {}/{}): {}", 
                    event.event_type, event.retry_count + 1, event.max_retries, e);
            }
        }
    }
    
    Ok(count)
}

async fn process_pending_heartbeats() -> anyhow::Result<usize> {
    let pending_heartbeats = offline_queue::get_pending_heartbeats().await?;
    let count = pending_heartbeats.len();
    
    for heartbeat in pending_heartbeats {
        // Try to send the heartbeat
        match super::send_heartbeat_to_backend(&heartbeat.heartbeat_data).await {
            Ok(_) => {
                // Mark as processed
                offline_queue::mark_heartbeat_processed(heartbeat.id).await?;
                log::debug!("✓ Sent queued heartbeat");
            }
            Err(e) => {
                // Mark as failed (increment retry count)
                offline_queue::mark_heartbeat_failed(heartbeat.id).await?;
                log::warn!("Failed to send queued heartbeat (retry {}/{}): {}", 
                    heartbeat.retry_count + 1, heartbeat.max_retries, e);
            }
        }
    }
    
    Ok(count)
}

