use tauri::AppHandle;
use tokio::time::Duration;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::OnceLock;

use crate::sampling::{idle_detector};
use crate::storage::{work_session, offline_queue};

use crate::commands::get_current_app;

// Global trigger to send immediate heartbeat
static IMMEDIATE_HEARTBEAT_TRIGGER: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

fn get_heartbeat_trigger() -> &'static Arc<Mutex<bool>> {
    IMMEDIATE_HEARTBEAT_TRIGGER.get_or_init(|| Arc::new(Mutex::new(false)))
}

/// Trigger an immediate heartbeat (called when app changes)
#[allow(dead_code)]
pub async fn trigger_immediate_heartbeat() {
    let trigger = get_heartbeat_trigger();
    let mut triggered = trigger.lock().await;
    *triggered = true;
    log::debug!("Immediate heartbeat triggered");
    crate::utils::logging::log_remote_non_blocking(
        "heartbeat_immediate_trigger",
        "debug",
        "Immediate heartbeat triggered",
        None
    ).await;
}

#[allow(dead_code)]
pub async fn start_heartbeat_service(_app_handle: AppHandle) {
    let interval_seconds = super::get_heartbeat_interval();
    let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
    let trigger = get_heartbeat_trigger();
    
    log::info!("Heartbeat service starting (interval: {}s)", interval_seconds);
    crate::utils::logging::log_remote_non_blocking(
        "heartbeat_service_start",
        "info",
        "Heartbeat service starting",
        Some(serde_json::json!({"interval_seconds": interval_seconds}))
    ).await;
    
    loop {
        // Wait for either the interval to tick or check for trigger periodically
        tokio::select! {
            _ = interval.tick() => {
                // Regular interval tick
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Check if immediate heartbeat was triggered
                let should_send_immediately = {
                    let mut triggered = trigger.lock().await;
                    if *triggered {
                        *triggered = false; // Reset trigger
                        true
                    } else {
                        false
                    }
                };
                
                if !should_send_immediately {
                    continue; // Nothing to do, loop again
                }
                // Otherwise, fall through to send heartbeat immediately
            }
        }
        
        // Check if services should continue running (authenticated AND clocked in)
        if !super::should_services_run().await {
            // Stop if user is not authenticated or not clocked in
            if !super::is_services_running().await {
                log::info!("Heartbeat service stopping - user clocked out or logged out");
                break; // Service stopped completely
            }
            // Otherwise, just wait before checking again
            continue;
        }

        // Send heartbeat - ALWAYS send, even when idle
        // The heartbeat includes the idle status, so the backend knows if user is active or idle
        log::debug!("Sending heartbeat");
        crate::utils::logging::log_remote_non_blocking(
            "heartbeat_send_attempt",
            "debug",
            "Attempting to send heartbeat",
            None
        ).await;
        
        match send_heartbeat().await {
            Ok(_) => {
                // Heartbeat sent successfully
                log::debug!("Heartbeat sent successfully");
                crate::utils::logging::log_remote_non_blocking(
                    "heartbeat_send_success",
                    "debug",
                    "Heartbeat sent successfully",
                    None
                ).await;
            }
            Err(e) => {
                log::error!("Failed to send heartbeat (will retry on next interval): {}", e);
                crate::utils::logging::log_remote_non_blocking(
                    "heartbeat_send_error",
                    "error",
                    "Failed to send heartbeat",
                    Some(serde_json::json!({"error": e.to_string()}))
                ).await;
                // Don't break - continue sending heartbeats on next interval
            }
        }
    }

    log::info!("Heartbeat service stopped");
}

#[allow(dead_code)]
async fn send_heartbeat() -> anyhow::Result<()> {
    // Get current app info
    let current_app = match get_current_app().await {
        Ok(app_opt) => app_opt,
        Err(e) => {
            log::debug!("Could not get current app for heartbeat: {}", e);
            None
        }
    };
    
    // Get idle time
    let idle_time = idle_detector::get_idle_time().await.unwrap_or(0);
    let idle_threshold = idle_detector::get_idle_threshold();
    let is_idle = idle_time >= idle_threshold;

    let now = chrono::Utc::now();
    
    // Check if there's an active work session
    let session_active = work_session::is_session_active().await.unwrap_or(false);
    
    let (session_start, total_session_time, total_active_today, total_idle_today) = if session_active {
        // Get session start time for time calculations
        let session_start = work_session::get_session_start_time().await.unwrap_or_else(|_| now);
        let total_session_time = (now - session_start).num_seconds();
        
        // Calculate cumulative active and idle time for today
        let (cumulative_active_time, cumulative_idle_time) = work_session::get_today_time_totals().await.unwrap_or((0, 0));
        
        // Add current session time to totals
        let current_session_active = if is_idle { 0 } else { total_session_time };
        let current_session_idle = if is_idle { total_session_time } else { 0 };
        
        let total_active_today = cumulative_active_time + current_session_active;
        let total_idle_today = cumulative_idle_time + current_session_idle;
        
        log::info!("📡 Heartbeat: user_state={} (backend_status=active), idle_time={}s, session={}s, active={}s, idle={}s", 
            if is_idle { "IDLE" } else { "ACTIVE" }, idle_time, total_session_time, total_active_today, total_idle_today);

        (session_start, total_session_time, total_active_today, total_idle_today)
    } else {
        // No active session - use default values
        log::debug!("Heartbeat: No active session");
        (now, 0, 0, 0)
    };

    // Create heartbeat data with complete time information
    // WORKAROUND: Always send status="active" to keep user in "Online Now" count
    // Backend should ideally treat both 'active' and 'idle' as online, but until then,
    // we send status="active" and let the backend/frontend use idle_time_seconds to show idle state
    let heartbeat_data = json!({
        "timestamp": now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "status": "active",  // Always "active" to stay in Online count (workaround)
        "idle_time_seconds": idle_time,  // Backend can use this to determine if user is idle
        "is_idle": is_idle,  // Explicit idle flag for future use
        "currentApp": current_app.as_ref().map(|app| json!({
            "name": app.name,
            "app_id": app.app_id,
            "window_title": app.window_title
        })),
        "session_start_time": session_start.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "total_session_time_seconds": total_session_time,
        "active_time_today_seconds": total_active_today,
        "idle_time_today_seconds": total_idle_today,
        "is_paused": super::is_services_paused().await
    });

    // Try to send heartbeat live first, fallback to queue if failed
    match super::send_heartbeat_to_backend(&heartbeat_data).await {
        Ok(_) => {
            log::info!("✓ Heartbeat sent (status=active, idle_time={}s, user_is_idle={})", 
                idle_time, is_idle);
            Ok(())
        }
        Err(e) => {
            log::warn!("Failed to send heartbeat live, queuing for later: {}", e);
            // Queue heartbeat for offline processing
            match offline_queue::queue_heartbeat(&heartbeat_data).await {
                Ok(_) => {
                    log::debug!("Heartbeat queued for later delivery");
                    Ok(())
                }
                Err(queue_err) => {
                    log::error!("Failed to queue heartbeat: {}", queue_err);
                    // Don't return error - we want heartbeat service to continue
                    // Just log the error and move on to next heartbeat
                    Ok(())
                }
            }
        }
    }
}

