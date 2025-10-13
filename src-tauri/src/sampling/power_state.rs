// Power state monitoring module for detecting sleep/wake events
use std::sync::atomic::{AtomicU64, Ordering};
use chrono::Utc;

// Track the last activity timestamp
static LAST_ACTIVITY_TIMESTAMP: AtomicU64 = AtomicU64::new(0);

// Store sleep start time
static SLEEP_START_TIME: AtomicU64 = AtomicU64::new(0);

// Track if system is currently sleeping
static IS_SLEEPING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Initialize power state monitoring
pub fn init() {
    let now = Utc::now().timestamp() as u64;
    LAST_ACTIVITY_TIMESTAMP.store(now, Ordering::Relaxed);
    log::info!("Power state monitoring initialized");
}

/// Update the last activity timestamp
pub fn update_last_activity() {
    let now = Utc::now().timestamp() as u64;
    LAST_ACTIVITY_TIMESTAMP.store(now, Ordering::Relaxed);
}

/// Get the last activity timestamp
#[allow(dead_code)]
pub fn get_last_activity_timestamp() -> u64 {
    LAST_ACTIVITY_TIMESTAMP.load(Ordering::Relaxed)
}

/// Mark system as entering sleep
#[allow(dead_code)]
pub fn mark_sleep_start() {
    let now = Utc::now().timestamp() as u64;
    SLEEP_START_TIME.store(now, Ordering::Relaxed);
    IS_SLEEPING.store(true, Ordering::Relaxed);
    log::info!("System entering sleep mode at {}", now);
}

/// Mark system as waking up and return sleep duration
pub fn mark_wake_up() -> u64 {
    let sleep_start = SLEEP_START_TIME.load(Ordering::Relaxed);
    let now = Utc::now().timestamp() as u64;
    let sleep_duration = if sleep_start > 0 {
        now.saturating_sub(sleep_start)
    } else {
        0
    };
    
    IS_SLEEPING.store(false, Ordering::Relaxed);
    SLEEP_START_TIME.store(0, Ordering::Relaxed);
    
    log::info!("System waking up, slept for {} seconds", sleep_duration);
    sleep_duration
}

/// Check if system is currently sleeping
pub fn is_system_sleeping() -> bool {
    IS_SLEEPING.load(Ordering::Relaxed)
}

/// Detect potential sleep by checking for large time gaps
/// Returns Some(gap_seconds) if a significant gap is detected, None otherwise
#[allow(dead_code)]
pub async fn detect_time_gap() -> Option<u64> {
    let last_activity = get_last_activity_timestamp();
    let now = Utc::now().timestamp() as u64;
    
    // If more than 10 minutes have passed since last activity, consider it a sleep event
    const SLEEP_THRESHOLD: u64 = 600; // 10 minutes
    
    if last_activity > 0 {
        let gap = now.saturating_sub(last_activity);
        if gap > SLEEP_THRESHOLD {
            log::warn!("Detected large time gap of {} seconds, system may have been sleeping", gap);
            return Some(gap);
        }
    }
    
    None
}

#[cfg(target_os = "windows")]
pub mod windows {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    lazy_static::lazy_static! {
        static ref POWER_CALLBACKS: Arc<RwLock<Vec<Box<dyn Fn(bool) + Send + Sync>>>> = 
            Arc::new(RwLock::new(Vec::new()));
    }
    
    /// Register a callback for power state changes
    #[allow(dead_code)]
    pub async fn register_power_callback<F>(callback: F) 
    where 
        F: Fn(bool) + Send + Sync + 'static 
    {
        let mut callbacks = POWER_CALLBACKS.write().await;
        callbacks.push(Box::new(callback));
    }
    
    /// Notify all registered callbacks
    #[allow(dead_code)]
    async fn notify_power_change(is_sleeping: bool) {
        let callbacks = POWER_CALLBACKS.read().await;
        for callback in callbacks.iter() {
            callback(is_sleeping);
        }
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    lazy_static::lazy_static! {
        static ref POWER_CALLBACKS: Arc<RwLock<Vec<Box<dyn Fn(bool) + Send + Sync>>>> = 
            Arc::new(RwLock::new(Vec::new()));
    }
    
    /// Register a callback for power state changes
    #[allow(dead_code)]
    pub async fn register_power_callback<F>(callback: F) 
    where 
        F: Fn(bool) + Send + Sync + 'static 
    {
        let mut callbacks = POWER_CALLBACKS.write().await;
        callbacks.push(Box::new(callback));
    }
    
    /// Notify all registered callbacks
    #[allow(dead_code)]
    async fn notify_power_change(is_sleeping: bool) {
        let callbacks = POWER_CALLBACKS.read().await;
        for callback in callbacks.iter() {
            callback(is_sleeping);
        }
    }
}

/// Start monitoring power state changes
#[allow(dead_code)]
pub async fn start_power_monitoring() {
    log::info!("Starting power state monitoring service");
    
    // Initialize power state
    init();
    
    // Start the time gap detection loop
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Update last activity timestamp
            update_last_activity();
            
            // Check for time gaps that might indicate sleep
            if let Some(gap) = detect_time_gap().await {
                log::warn!("Detected potential sleep event with gap of {} seconds", gap);
                
                // If we weren't already marked as sleeping, handle the wake event
                if !is_system_sleeping() {
                    handle_system_wake(gap).await;
                }
            }
        }
    });
}

/// Handle system sleep event
#[allow(dead_code)]
pub async fn handle_system_sleep() {
    if is_system_sleeping() {
        return; // Already in sleep state
    }
    
    mark_sleep_start();
    log::info!("🌙 System is going to sleep");
    
    // Send idle_start event
    let event_data = serde_json::json!({
        "reason": "system_sleep",
        "timestamp": Utc::now().to_rfc3339(),
        "idle_time_seconds": 0,
    });
    
    if let Err(e) = crate::sampling::send_event_to_backend("idle_start", &event_data).await {
        log::error!("Failed to send sleep idle_start event: {}", e);
        // Queue the event for later
        if let Err(e) = crate::storage::offline_queue::queue_event("idle_start", &event_data).await {
            log::error!("Failed to queue sleep event: {}", e);
        }
    }
}

/// Handle system wake event
pub async fn handle_system_wake(sleep_duration: u64) {
    if !is_system_sleeping() && sleep_duration == 0 {
        return; // Not coming from sleep
    }
    
    let actual_duration = if sleep_duration > 0 {
        sleep_duration
    } else {
        mark_wake_up()
    };
    
    log::info!("☀️ System woke up after {} seconds", actual_duration);
    
    // Send idle_end event with the sleep duration
    let event_data = serde_json::json!({
        "reason": "system_wake",
        "timestamp": Utc::now().to_rfc3339(),
        "idle_time_seconds": actual_duration,
        "sleep_duration_seconds": actual_duration,
    });
    
    if let Err(e) = crate::sampling::send_event_to_backend("idle_end", &event_data).await {
        log::error!("Failed to send wake idle_end event: {}", e);
        // Queue the event for later
        if let Err(e) = crate::storage::offline_queue::queue_event("idle_end", &event_data).await {
            log::error!("Failed to queue wake event: {}", e);
        }
    }
    
    // Update app usage to reflect the idle time
    if let Err(e) = crate::storage::app_usage::handle_system_wake(actual_duration).await {
        log::error!("Failed to update app usage after wake: {}", e);
    }
}

