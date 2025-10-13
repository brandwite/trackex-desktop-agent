// Sampling module - simplified for production testing

pub mod app_focus;
pub mod idle_detector;
pub mod heartbeat;
pub mod power_state;

#[allow(dead_code)]
pub fn is_dev_mode() -> bool {
    std::env::var("TRACKEX_DEV_SHORT_INTERVALS").is_ok()
}

#[allow(dead_code)]
pub fn get_app_focus_interval() -> u64 {
    if is_dev_mode() {
        1 // 1 second for development
    } else {
        3 // 3 seconds for production
    }
}

#[allow(dead_code)]
pub fn get_heartbeat_interval() -> u64 {
    if is_dev_mode() {
        10 // 10 seconds for development
    } else {
        30 // 30 seconds for production
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use crate::storage::offline_queue;

// Global state for background services
static SERVICES_RUNNING: AtomicBool = AtomicBool::new(false);
static SERVICES_PAUSED: AtomicBool = AtomicBool::new(false);

// Helper function to check if user is authenticated
#[allow(dead_code)]
pub async fn is_authenticated() -> bool {
    crate::storage::get_device_token().await.is_ok_and(|token| !token.is_empty())
}

// Helper function to check if user is clocked in (has active work session)
#[allow(dead_code)]
pub async fn is_clocked_in() -> bool {
    crate::storage::work_session::is_session_active().await.unwrap_or(false)
}

// Helper function to check if services should be running
// Services should only run when user is authenticated AND clocked in
#[allow(dead_code)]
pub async fn should_services_run() -> bool {
    let authenticated = is_authenticated().await;
    let clocked_in = is_clocked_in().await;
    let running = is_services_running().await;
    let paused = is_services_paused().await;
    
    let should_run = authenticated && clocked_in && running && !paused;
    
    // Log the decision for debugging
    log::debug!("Service check: auth={}, clocked_in={}, running={}, paused={}, should_run={}", 
        authenticated, clocked_in, running, paused, should_run);
    
    should_run
}

lazy_static::lazy_static! {
    static ref BACKGROUND_SERVICES: RwLock<BackgroundServiceState> = 
        RwLock::new(BackgroundServiceState::new());
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackgroundServiceState {
    pub app_focus_running: bool,
    pub heartbeat_running: bool,
    pub idle_detection_running: bool,
    pub last_app_check: Option<chrono::DateTime<chrono::Utc>>,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub last_idle_check: Option<chrono::DateTime<chrono::Utc>>,
}

impl BackgroundServiceState {
    pub fn new() -> Self {
        Self {
            app_focus_running: false,
            heartbeat_running: false,
            idle_detection_running: false,
            last_app_check: None,
            last_heartbeat: None,
            last_idle_check: None,
        }
    }
}

#[allow(dead_code)]
pub async fn is_services_running() -> bool {
    SERVICES_RUNNING.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub async fn is_services_paused() -> bool {
    SERVICES_PAUSED.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub async fn start_services() {
    SERVICES_RUNNING.store(true, Ordering::Relaxed);
    SERVICES_PAUSED.store(false, Ordering::Relaxed);
}


#[allow(dead_code)]
pub async fn stop_services() {
    SERVICES_RUNNING.store(false, Ordering::Relaxed);
}

#[allow(dead_code)]
pub async fn pause_services() {
    SERVICES_PAUSED.store(true, Ordering::Relaxed);
}

#[allow(dead_code)]
pub async fn resume_services() {
    SERVICES_PAUSED.store(false, Ordering::Relaxed);
}

#[allow(dead_code)]
pub async fn get_service_state() -> BackgroundServiceState {
    let state = BACKGROUND_SERVICES.read().await;
    state.clone()
}

#[allow(dead_code)]
pub async fn update_service_state<F>(updater: F) 
where 
    F: FnOnce(&mut BackgroundServiceState),
{
    let mut state = BACKGROUND_SERVICES.write().await;
    updater(&mut state);
}

#[allow(dead_code)]
pub async fn start_all_background_services(app_handle: tauri::AppHandle) {
    
    // Start services
    start_services().await;
    
    // Start app focus sampling
    let app_handle1 = app_handle.clone();
    tokio::spawn(async move {
        update_service_state(|state| {
            state.app_focus_running = true;
            state.last_app_check = Some(chrono::Utc::now());
        }).await;
        
        app_focus::start_sampling(app_handle1).await;
        
        update_service_state(|state| {
            state.app_focus_running = false;
        }).await;
    });
    
    // Start heartbeat service
    let app_handle2 = app_handle.clone();
    tokio::spawn(async move {
        update_service_state(|state| {
            state.heartbeat_running = true;
            state.last_heartbeat = Some(chrono::Utc::now());
        }).await;
        
        heartbeat::start_heartbeat_service(app_handle2).await;
        
        update_service_state(|state| {
            state.heartbeat_running = false;
        }).await;
    });
    
    // Start idle detection service
    let app_handle3 = app_handle.clone();
    tokio::spawn(async move {
        update_service_state(|state| {
            state.idle_detection_running = true;
            state.last_idle_check = Some(chrono::Utc::now());
        }).await;
        
        start_idle_detection_service(app_handle3).await;
        
        update_service_state(|state| {
            state.idle_detection_running = false;
        }).await;
    });
    
    // Start job polling
    tokio::spawn(async move {
        crate::api::job_polling::start_job_polling(app_handle).await;
    });
    
}

// Global idle state tracking
static mut LAST_IDLE_STATE: bool = false;
static mut IDLE_STATE_INITIALIZED: bool = false;

#[allow(dead_code)]
async fn start_idle_detection_service(_app_handle: tauri::AppHandle) {
    let interval_seconds = 5; // Check idle status every 5 seconds

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
    let mut last_check_time = chrono::Utc::now();
    
    loop {
        // Check if services should continue running (authenticated AND clocked in)
        if !should_services_run().await {
            // Stop if user is not authenticated or not clocked in
            if !is_services_running().await {
                break; // Service stopped completely
            }
            // Reset idle state when not running
            unsafe {
                IDLE_STATE_INITIALIZED = false;
            }
            // Otherwise, just wait before checking again
            interval.tick().await;
            continue;
        }

        // Detect potential sleep/wake events by checking for large time gaps
        let now = chrono::Utc::now();
        let time_since_last_check = (now - last_check_time).num_seconds() as u64;
        
        // If more than 2x the interval has passed, we likely woke from sleep
        if time_since_last_check > (interval_seconds * 3) {
            log::warn!("⏰ Detected large time gap of {} seconds - system may have been sleeping", time_since_last_check);
            power_state::handle_system_wake(time_since_last_check).await;
            
            // Reset idle state after wake
            unsafe {
                IDLE_STATE_INITIALIZED = false;
            }
        }
        
        last_check_time = now;
        power_state::update_last_activity();

        // Run idle detection (only when authenticated and clocked in)
        // Update service state
        update_service_state(|state| {
            state.last_idle_check = Some(chrono::Utc::now());
        }).await;
        
        // Check idle status and send events if needed
        if let Ok(idle_time) = idle_detector::get_idle_time().await {
            let threshold = idle_detector::get_idle_threshold();
            let is_idle = idle_time >= threshold;
            
            // Check if idle state has changed
            let state_changed = unsafe {
                if !IDLE_STATE_INITIALIZED {
                    IDLE_STATE_INITIALIZED = true;
                    LAST_IDLE_STATE = is_idle;
                    false // Don't send event on first check
                } else if LAST_IDLE_STATE != is_idle {
                    LAST_IDLE_STATE = is_idle;
                    true
                } else {
                    false
                }
            };
            
            // Update current app usage session with idle status
            if let Err(e) = crate::storage::app_usage::update_current_session(is_idle).await {
                log::error!("Failed to update app session idle status: {}", e);
            }
            
            // Send idle events only when status changes
            if state_changed {
                let event_type = if is_idle { "idle_start" } else { "idle_end" };
                let event_data = serde_json::json!({
                    "idle_time_seconds": idle_time,
                    "threshold_seconds": threshold,
                    "is_idle": is_idle,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "reason": "user_activity"
                });
                log::debug!("Sending idle event: {} (idle_time: {}s)", event_type, idle_time);
                // Try to send live first, fallback to queue if failed
                match send_event_to_backend(event_type, &event_data).await {
                    Ok(_) => {
                        log::debug!("✓ Idle event sent successfully");
                    }
                    Err(e) => {
                        log::warn!("🔍 Failed to send idle event live, queuing for later: {}", e);
                        if let Err(e) = crate::storage::offline_queue::queue_event(event_type, &event_data).await {
                            log::error!("Failed to queue idle event: {}", e);
                        }
                    }
                }
            }
        }

        interval.tick().await;
    }

}

#[allow(dead_code)]
pub fn get_job_polling_interval() -> u64 {
    if is_dev_mode() {
        5 // 5 seconds for development
    } else {
        10 // 10 seconds for production (faster screenshot response)
    }
}

// Queue processing service
#[allow(dead_code)]
pub async fn start_queue_processing_service() {
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    
    loop {
        if !SERVICES_RUNNING.load(Ordering::Relaxed) {
            break;
        }

        // Only process queue when authenticated
        if !is_authenticated().await {
            interval.tick().await;
            continue;
        }

        // Process pending heartbeats
        if let Ok(heartbeats) = offline_queue::get_pending_heartbeats().await {
            if !heartbeats.is_empty() {
            }
            for heartbeat in heartbeats {
                if let Err(e) = send_heartbeat_to_backend(&heartbeat.heartbeat_data).await {
                    log::error!("Failed to send heartbeat4: {}", e);
                    if let Err(e) = offline_queue::mark_heartbeat_failed(heartbeat.id).await {
                        log::error!("Failed to mark heartbeat as failed: {}", e);
                    }
                } else {
                    if let Err(e) = offline_queue::mark_heartbeat_processed(heartbeat.id).await {
                        log::error!("Failed to mark heartbeat as processed: {}", e);
                    }
                }
            }
        } else {
        }

        // Process pending events
        if let Ok(events) = offline_queue::get_pending_events().await {
            for event in events {
                log::debug!("Sending event: 1");
                if let Err(e) = send_event_to_backend(&event.event_type, &event.event_data).await {
                    log::error!("Failed to send event: {}", e);
                    if let Err(e) = offline_queue::mark_event_failed(event.id).await {
                        log::error!("Failed to mark event as failed: {}", e);
                    }
                } else {
                    if let Err(e) = offline_queue::mark_event_processed(event.id).await {
                        log::error!("Failed to mark event as processed: {}", e);
                    }
                }
            }
        }

        interval.tick().await;
    }

}

// Enhanced sync service that syncs all local data when reconnected
#[allow(dead_code)]
pub async fn start_sync_service() {
    
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    
    loop {
        if !SERVICES_RUNNING.load(Ordering::Relaxed) {
            break;
        }

        // Only sync when authenticated and online
        if !is_authenticated().await {
            interval.tick().await;
            continue;
        }

        // Check if we're online and have pending data to sync
        if is_online().await {
            
            // Sync pending heartbeats
            if let Ok(heartbeats) = offline_queue::get_pending_heartbeats().await {
                if !heartbeats.is_empty() {
                    for heartbeat in heartbeats {
                        if let Err(e) = send_heartbeat_to_backend(&heartbeat.heartbeat_data).await {
                            log::error!("Failed to sync heartbeat {}: {}", heartbeat.id, e);
                            if let Err(e) = offline_queue::mark_heartbeat_failed(heartbeat.id).await {
                                log::error!("Failed to mark heartbeat as failed: {}", e);
                            }
                        } else {
                            if let Err(e) = offline_queue::mark_heartbeat_processed(heartbeat.id).await {
                                log::error!("Failed to mark heartbeat as processed: {}", e);
                            }
                        }
                    }
                }
            }

            // Sync pending events
            if let Ok(events) = offline_queue::get_pending_events().await {
                if !events.is_empty() {
                    for event in events {
                        log::debug!("Sending event: {:?}", event);
                        if let Err(e) = send_event_to_backend(&event.event_type, &event.event_data).await {
                            log::error!("Failed to sync event {}: {}", event.id, e);
                            if let Err(e) = offline_queue::mark_event_failed(event.id).await {
                                log::error!("Failed to mark event as failed: {}", e);
                            }
                        } else {
                            if let Err(e) = offline_queue::mark_event_processed(event.id).await {
                                log::error!("Failed to mark event as processed: {}", e);
                            }
                        }
                    }
                }
            }

            // Skip syncing app_usage sessions - app_focus events already handle this
            // if let Err(e) = sync_local_app_usage_sessions().await {
            //     log::error!("Failed to sync local app usage sessions: {}", e);
            // }
        } else {
        }

        interval.tick().await;
    }

}

// Check if we're online by testing a simple API call
async fn is_online() -> bool {
    if let Ok(server_url) = crate::storage::get_server_url().await {
        if let Ok(device_token) = crate::storage::get_device_token().await {
            if !server_url.is_empty() && !device_token.is_empty() {
                let client = reqwest::Client::new();
                let test_url = format!("{}/api/auth/simple-session", server_url.trim_end_matches('/'));
                
                match client
                    .get(&test_url)
                    .header("Authorization", format!("Bearer {}", device_token))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await
                {
                    Ok(response) => return response.status().is_success(),
                    Err(_) => return false,
                }
            }
        }
    }
    false
}

// Removed sync_local_app_usage_sessions function - no longer needed
// App usage is now tracked solely via app_focus events, eliminating duplication

pub async fn send_heartbeat_to_backend(heartbeat_data: &serde_json::Value) -> anyhow::Result<()> {
    // Get server URL and device token from storage
    let server_url = crate::storage::get_server_url().await?;
    let device_token = crate::storage::get_device_token().await?;
    log::info!("Server URL: {}", server_url);
    log::info!("Device token: {}", device_token);
    if server_url.is_empty() || device_token.is_empty() {
        log::warn!("🔍 Cannot send heartbeat: server_url or device_token is empty");
        return Ok(());
    }
    
    // Log the heartbeat data being sent
    
    let client = reqwest::Client::new();
    let heartbeat_url = format!("{}/api/ingest/heartbeat", server_url.trim_end_matches('/'));
    
    
    let response = client
        .post(&heartbeat_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", device_token))
        .json(heartbeat_data)
        .send()
        .await?;
    
    
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        log::error!("🔍 Heartbeat failed with status {}: {}", status, text);
        Err(anyhow::anyhow!("Heartbeat failed with status {}: {}", status, text))
    }
}

pub async fn send_event_to_backend(event_type: &str, event_data: &serde_json::Value) -> anyhow::Result<()> {
    // Get server URL and device token from storage
    let server_url = crate::storage::get_server_url().await?;
    let device_token = crate::storage::get_device_token().await?;
    
    if server_url.is_empty() || device_token.is_empty() {
        return Ok(());
    }
    
    let client = reqwest::Client::new();
    let events_url = format!("{}/api/ingest/events", server_url.trim_end_matches('/'));
    
    let event_payload = serde_json::json!({
        "events": [{
            "type": event_type,
            "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            "data": event_data,
            "from": "send_event_to_backend"
        }]
    });
    
    let response = client
        .post(&events_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", device_token))
        .json(&event_payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(anyhow::anyhow!("Event failed with status {}: {}", status, text))
    }
}