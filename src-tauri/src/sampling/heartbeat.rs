use tauri::AppHandle;
use tokio::time::Duration;
use serde_json::json;

use crate::sampling::{idle_detector};
use crate::storage::{work_session, offline_queue};

use crate::commands::get_current_app;

#[allow(dead_code)]
pub async fn start_heartbeat_service(_app_handle: AppHandle) {
    let interval_seconds = super::get_heartbeat_interval();

    let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
    
    loop {
        // Check if services should continue running (authenticated AND clocked in)
        if !super::should_services_run().await {
            // Stop if user is not authenticated or not clocked in
            if !super::is_services_running().await {
                break; // Service stopped completely
            }
            // Otherwise, just wait before checking again
            interval.tick().await;
            continue;
        }

        // Send heartbeat
        if let Err(e) = send_heartbeat().await {
            log::error!("Failed to send heartbeat: {}", e);
        }

        interval.tick().await;
    }

}

#[allow(dead_code)]
async fn send_heartbeat() -> anyhow::Result<()> {
    // Get current app info
    let current_app = get_current_app().await.ok();
    
    // Get idle time
    let idle_time = idle_detector::get_idle_time().await.unwrap_or(0);
    let idle_threshold = idle_detector::get_idle_threshold();
    let is_idle = idle_time >= idle_threshold;

    let now = chrono::Utc::now();
    
    // Get session start time for time calculations
    let session_start = work_session::get_session_start_time().await.unwrap_or_else(|_| now);
    let total_session_time = (now - session_start).num_seconds();
    
    // Note: We no longer send time calculations in heartbeats
    // Time totals are calculated from the database in the Live View API

    // Log the session info for debugging

    // Create heartbeat data - only send current status and app info
    let heartbeat_data = json!({
        "timestamp": now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "status": if is_idle { "idle" } else { "active" },
        "currentApp": current_app.map(|app| json!({
            "name": app.clone().unwrap().name,
            "app_id": app.clone().unwrap().app_id,
            "window_title": app.clone().unwrap().window_title
        })),
        "session_start_time": session_start.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "total_session_time_seconds": total_session_time,
        // Removed: active_time_today_seconds and idle_time_today_seconds
        // These are now calculated from the database in the Live View API
        "is_paused": super::is_services_paused().await
    });

    // Log the complete heartbeat payload for debugging

    // Try to send heartbeat live first, fallback to queue if failed
    match super::send_heartbeat_to_backend(&heartbeat_data).await {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            log::warn!("🔍 Failed to send heartbeat live, queuing for later: {}", e);
            // Queue heartbeat for offline processing
            offline_queue::queue_heartbeat(&heartbeat_data).await?;
            Ok(())
        }
    }
}

