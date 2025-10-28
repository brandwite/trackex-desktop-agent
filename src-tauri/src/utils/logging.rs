use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::collections::HashSet;
use std::sync::LazyLock;

// Global configuration for remote logging
static REMOTE_LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static ALLOWED_LEVELS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn init() {
    let mut builder = Builder::from_default_env();
    
    builder
        .target(Target::Stdout)
        .filter_level(LevelFilter::Error) // Only show errors by default
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .init();

    // Initialize remote logging configuration
    init_remote_logging_config();
}

/// Initialize remote logging configuration
fn init_remote_logging_config() {
    // Check if debug mode is enabled via environment variable
    let debug_mode = std::env::var("TRACKEX_DEBUG_MODE").unwrap_or_default() == "true";
    DEBUG_MODE.store(debug_mode, Ordering::Relaxed);
    
    // Check if remote logging is enabled via environment variable
    let remote_enabled = std::env::var("TRACKEX_REMOTE_LOGGING").unwrap_or_default() == "true";
    REMOTE_LOGGING_ENABLED.store(remote_enabled, Ordering::Relaxed);
    
    // Parse allowed log levels from environment variable
    let levels_str = std::env::var("TRACKEX_LOG_LEVELS").unwrap_or_default();
    let mut allowed_levels = HashSet::new();
    
    if !levels_str.is_empty() {
        for level in levels_str.split(',') {
            allowed_levels.insert(level.trim().to_lowercase());
        }
    } else {
        // Default: only allow error level
        allowed_levels.insert("error".to_string());
    }
    
    if let Ok(mut levels) = ALLOWED_LEVELS.lock() {
        *levels = allowed_levels;
    }
    
    log::info!("Remote logging config: enabled={}, debug_mode={}, levels={:?}", 
        remote_enabled, debug_mode, levels_str);
}

/// Check if remote logging should be enabled based on configuration
fn should_send_remote_log(level: &str) -> bool {
    // Check if remote logging is enabled
    if !REMOTE_LOGGING_ENABLED.load(Ordering::Relaxed) {
        return false;
    }
    
    // Check if debug mode is required and enabled
    if level == "debug" && !DEBUG_MODE.load(Ordering::Relaxed) {
        return false;
    }
    
    // Check if the level is in the allowed levels
    if let Ok(allowed_levels) = ALLOWED_LEVELS.lock() {
        allowed_levels.contains(&level.to_lowercase())
    } else {
        false
    }
}

/// Send a small JSON log to remote endpoint (fire-and-forget)
/// This function never panics and will not block the main loop.
/// Only sends logs if remote logging is enabled and the level is allowed.
pub async fn log_remote_non_blocking(event: &str, level: &str, message: &str, context: Option<serde_json::Value>) {
    // Check if we should send this log remotely
    if !should_send_remote_log(level) {
        return;
    }
    
    // Build payload
    let payload = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "event": event,
        "level": level,
        "message": message,
        "context": context.unwrap_or(serde_json::json!({}))
    });

    // Spawn and detach the network call with very short timeout
    tokio::spawn(async move {
        // Very short timeout client to prevent hanging
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(500))
            .build() {
                Ok(c) => c,
                Err(_) => {
                    // Silently fail - don't log client creation errors
                    return;
                }
            };

        // Resolve server URL from storage (falls back internally to default)
        let base_url = match crate::storage::get_server_url().await {
            Ok(u) => u,
            Err(_) => {
                // Silently fail - don't log server URL errors
                return;
            }
        };
        let base = base_url.trim_end_matches('/');
        let url = format!("{}/api/logs", base);
        
        // Use a timeout wrapper to ensure we don't hang
        match tokio::time::timeout(std::time::Duration::from_millis(300), async {
            client
                .post(url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
        }).await {
            Ok(Ok(resp)) => {
                if !resp.status().is_success() {
                    // Silently ignore failed responses
                }
            }
            Ok(Err(_)) => {
                // Silently ignore network errors
            }
            Err(_) => {
                // Timeout occurred, silently ignore
            }
        }
    });
}

/// Update remote logging configuration at runtime
pub fn update_remote_logging_config(enabled: bool, debug_mode: bool, allowed_levels: Vec<String>) {
    REMOTE_LOGGING_ENABLED.store(enabled, Ordering::Relaxed);
    DEBUG_MODE.store(debug_mode, Ordering::Relaxed);
    
    if let Ok(mut levels) = ALLOWED_LEVELS.lock() {
        levels.clear();
        for level in allowed_levels.iter() {
            levels.insert(level.to_lowercase());
        }
    }
    
    log::info!("Remote logging config updated: enabled={}, debug_mode={}, levels={:?}", 
        enabled, debug_mode, allowed_levels.clone());
}

/// Get current remote logging configuration
pub fn get_remote_logging_config() -> (bool, bool, Vec<String>) {
    let enabled = REMOTE_LOGGING_ENABLED.load(Ordering::Relaxed);
    let debug_mode = DEBUG_MODE.load(Ordering::Relaxed);
    let levels = if let Ok(allowed_levels) = ALLOWED_LEVELS.lock() {
        allowed_levels.iter().cloned().collect()
    } else {
        Vec::new()
    };
    
    (enabled, debug_mode, levels)
}

/// Fetch logging configuration from backend API
pub async fn fetch_logging_config_from_backend() -> Result<(), String> {
    // Get server URL
    let server_url = match crate::storage::get_server_url().await {
        Ok(url) => url,
        Err(e) => {
            log::warn!("Failed to get server URL for logging config: {}", e);
            return Ok(());
        }
    };

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let config_url = format!("{}/api/system/logging-config", server_url.trim_end_matches('/'));
    
    log::info!("🔍 Fetching global logging configuration from: {}", config_url);

    match client
        .get(&config_url)
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(config) => {
                        log::info!("✅ Successfully fetched logging configuration from backend");
                        
                        // Parse and apply the configuration
                        let enabled = config.get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        let debug_mode = config.get("debug_mode")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        
                        let allowed_levels = config.get("allowed_levels")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect::<Vec<String>>())
                            .unwrap_or_else(|| vec!["error".to_string()]);
                        
                        // Apply the configuration
                        let levels_for_log = allowed_levels.clone();
                        update_remote_logging_config(enabled, debug_mode, allowed_levels);
                        
                        log::info!("📝 Applied remote logging config: enabled={}, debug_mode={}, levels={:?}", 
                            enabled, debug_mode, levels_for_log);
                    }
                    Err(e) => {
                        log::warn!("❌ Failed to parse logging configuration response: {}", e);
                    }
                }
            } else {
                log::warn!("❌ Backend returned error status {} for logging config", response.status());
            }
        }
        Err(e) => {
            log::debug!("🔍 Failed to fetch logging configuration from backend: {}", e);
            // Don't treat this as an error - backend might not have this endpoint yet
        }
    }

    Ok(())
}

/// Start periodic sync service for logging configuration
pub async fn start_logging_config_sync_service() {
    log::info!("🔄 Starting global logging configuration sync service");
    
    tokio::spawn(async {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            // Sync global logging configuration (no authentication required)
            if let Err(e) = fetch_logging_config_from_backend().await {
                log::debug!("Failed to sync global logging configuration: {}", e);
            }
        }
    });
}

/// Force sync logging configuration (for manual triggers)
pub async fn sync_logging_config_now() -> Result<(), String> {
    fetch_logging_config_from_backend().await
}