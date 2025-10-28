use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::storage::{AppState, consent, app_usage};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub server_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentStatus {
    pub accepted: bool,
    pub accepted_at: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSessionInfo {
    pub is_active: bool,
    pub started_at: Option<String>,
    pub current_app: Option<String>,
    pub idle_time_seconds: u64,
    pub is_paused: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingStatus {
    pub is_tracking: bool,
    pub is_paused: bool,
    pub current_app: Option<String>,
    pub idle_time_seconds: u64,
}

// Use AppInfo from the app_focus module
use crate::sampling::app_focus::AppInfo;

// Helper functions for device registration
fn get_platform_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "macOS", 
        "linux" => "Linux",
        _ => "Unknown"
    }
}

fn get_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        // Method 1: Try PowerShell to get accurate Windows version (most reliable)
        if let Ok(output) = std::process::Command::new("powershell")
            .args(&["-Command", "Get-ComputerInfo | Select-Object WindowsProductName, WindowsVersion, WindowsBuildLabEx | ConvertTo-Json"])
            .output()
        {
            if let Ok(json_str) = String::from_utf8(output.stdout) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(product_name) = json.get("WindowsProductName").and_then(|v| v.as_str()) {
                        if let Some(version) = json.get("WindowsVersion").and_then(|v| v.as_str()) {
                            let result = format!("{} {}", product_name, version);
                            return result;
                        }
                    }
                }
            }
        }
        
        // Method 2: Try wmic (Windows Management Instrumentation)
        if let Ok(output) = std::process::Command::new("wmic")
            .args(&["os", "get", "Caption,Version,BuildNumber", "/value"])
            .output()
        {
            if let Ok(wmic_output) = String::from_utf8(output.stdout) {
                let mut caption = String::new();
                let mut version = String::new();
                let mut build = String::new();
                
                for line in wmic_output.lines() {
                    if line.starts_with("Caption=") {
                        caption = line.replace("Caption=", "").trim().to_string();
                    } else if line.starts_with("Version=") {
                        version = line.replace("Version=", "").trim().to_string();
                    } else if line.starts_with("BuildNumber=") {
                        build = line.replace("BuildNumber=", "").trim().to_string();
                    }
                }
                
                if !caption.is_empty() && !version.is_empty() {
                    let result = format!("{} Version {} Build {}", caption, version, build);
                    return result;
                }
            }
        }
        
        // Method 3: Parse cmd /C ver output (improved parsing)
        if let Ok(output) = std::process::Command::new("cmd")
            .args(&["/C", "ver"])
            .output()
        {
            if let Ok(version_str) = String::from_utf8(output.stdout) {
                let trimmed = version_str.trim();
                
                // Parse "Microsoft Windows [Version 10.0.22621.2861]"
                if let Some(start) = trimmed.find("[Version ") {
                    if let Some(end) = trimmed[start..].find(']') {
                        let version_part = &trimmed[start + 9..start + end];
                        let parts: Vec<&str> = version_part.split('.').collect();
                        if parts.len() >= 4 {
                            let major = parts[0].parse::<u32>().unwrap_or(0);
                            let minor = parts[1].parse::<u32>().unwrap_or(0);
                            let build = parts[2].parse::<u32>().unwrap_or(0);
                            let revision = parts[3].parse::<u32>().unwrap_or(0);
                            
                            // Better Windows version detection
                            let os_name = if major >= 10 && build >= 22000 {
                                "Windows 11"
                            } else if major >= 10 {
                                "Windows 10"
                            } else if major == 6 && minor == 3 {
                                "Windows 8.1"
                            } else if major == 6 && minor == 2 {
                                "Windows 8"
                            } else if major == 6 && minor == 1 {
                                "Windows 7"
                            } else {
                                "Windows"
                            };
                            
                            let result = format!("{} Build {}.{}.{}.{}", os_name, major, minor, build, revision);
                            return result;
                        }
                    }
                }
                return trimmed.to_string();
            }
        }
        
        // Method 4: Fallback to GetVersionExW (but with better build number detection)
        unsafe {
            use winapi::um::sysinfoapi::GetVersionExW;
            use winapi::um::winnt::OSVERSIONINFOW;
            use std::mem;
            
            let mut os_info: OSVERSIONINFOW = mem::zeroed();
            os_info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOW>() as u32;
            
            if GetVersionExW(&mut os_info) != 0 {
                let major = os_info.dwMajorVersion;
                let minor = os_info.dwMinorVersion;
                let build = os_info.dwBuildNumber;
                
                
                // Use build number to better detect Windows 10/11
                let os_name = if build >= 22000 {
                    "Windows 11"  // Windows 11 builds start from 22000
                } else if build >= 10240 {
                    "Windows 10"  // Windows 10 builds start from 10240
                } else if major == 6 && minor == 3 {
                    "Windows 8.1"
                } else if major == 6 && minor == 2 {
                    "Windows 8"
                } else if major == 6 && minor == 1 {
                    "Windows 7"
                } else {
                    "Windows"
                };
                
                let result = format!("{} Build {}", os_name, build);
                return result;
            }
        }
        
        log::warn!("🔍 All Windows version detection methods failed");
        "Windows Unknown".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        // Get macOS version using sw_vers
        if let Ok(output) = std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
        {
            if let Ok(version_str) = String::from_utf8(output.stdout) {
                return format!("macOS {}", version_str.trim());
            }
        }
        
        // Fallback to system_profiler
        if let Ok(output) = std::process::Command::new("system_profiler")
            .args(&["SPSoftwareDataType"])
            .output()
        {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    if line.contains("System Version:") {
                        return line.replace("System Version:", "").trim().to_string();
                    }
                }
            }
        }
        
        "macOS Unknown".to_string()
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try to get Linux version from /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line.replace("PRETTY_NAME=", "").trim_matches('"').to_string();
                }
            }
        }
        
        // Fallback to uname
        if let Ok(output) = std::process::Command::new("uname")
            .args(&["-r"])
            .output()
        {
            if let Ok(kernel_version) = String::from_utf8(output.stdout) {
                return format!("Linux Kernel {}", kernel_version.trim());
            }
        }
        
        "Linux Unknown".to_string()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        format!("{} Unknown", std::env::consts::OS)
    }
}

fn get_device_name() -> String {
    #[cfg(target_os = "windows")]
    {
        // Try to get the computer name on Windows with consistent fallbacks
        use std::process::Command;
        
        // Method 1: Try hostname command
        if let Ok(output) = Command::new("hostname").output() {
            if let Ok(hostname) = String::from_utf8(output.stdout) {
                let trimmed = hostname.trim().to_string();
                if !trimmed.is_empty() && trimmed != "hostname" {
                    return trimmed;
                }
            }
        }
        
        // Method 2: Try COMPUTERNAME environment variable
        if let Ok(computername) = std::env::var("COMPUTERNAME") {
            let trimmed = computername.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        
        // Method 3: Try USERNAME as last resort
        if let Ok(username) = std::env::var("USERNAME") {
            let trimmed = username.trim().to_string();
            if !trimmed.is_empty() {
                return format!("Windows-{}", trimmed);
            }
        }
        
        // Final fallback
        log::warn!("🔍 All Windows device name methods failed, using fallback");
        "Windows-Unknown".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        // Try to get the computer name on macOS
        use std::process::Command;
        if let Ok(output) = Command::new("scutil").args(&["--get", "ComputerName"]).output() {
            if let Ok(hostname) = String::from_utf8(output.stdout) {
                return hostname.trim().to_string();
            }
        }
        // Fallback to environment variable
        return std::env::var("HOSTNAME").unwrap_or_else(|_| "macOS Device".to_string());
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try to get the hostname on Linux
        use std::process::Command;
        if let Ok(output) = Command::new("hostname").output() {
            if let Ok(hostname) = String::from_utf8(output.stdout) {
                return hostname.trim().to_string();
            }
        }
        // Fallback to environment variable
        return std::env::var("HOSTNAME").unwrap_or_else(|_| "Linux Device".to_string());
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Fallback for unknown platforms
        "Unknown Device".to_string()
    }
}

// Import PermissionsStatus from our dedicated permissions module
use crate::permissions::PermissionsStatus;

#[tauri::command]
pub async fn trigger_sync() -> Result<String, String> {
    
    // Try to sync pending heartbeats
    let mut synced_heartbeats = 0;
    if let Ok(heartbeats) = crate::storage::offline_queue::get_pending_heartbeats().await {
        for heartbeat in heartbeats {
            if let Ok(_) = crate::sampling::send_heartbeat_to_backend(&heartbeat.heartbeat_data).await {
                if let Ok(_) = crate::storage::offline_queue::mark_heartbeat_processed(heartbeat.id).await {
                    synced_heartbeats += 1;
                }
            }
        }
    }
    
    // Try to sync pending events
    let mut synced_events = 0;
    if let Ok(events) = crate::storage::offline_queue::get_pending_events().await {
        for event in events {
            if let Ok(_) = crate::sampling::send_event_to_backend(&event.event_type, &event.event_data).await {
                if let Ok(_) = crate::storage::offline_queue::mark_event_processed(event.id).await {
                    synced_events += 1;
                }
            }
        }
    }
    
    let message = format!("Sync completed: {} heartbeats, {} events synced", synced_heartbeats, synced_events);
    Ok(message)
}

#[tauri::command]
pub async fn login(
    request: LoginRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
    _app_handle: tauri::AppHandle,
) -> Result<AuthStatus, String> {
    
    crate::utils::logging::log_remote_non_blocking(
        "login_start",
        "info",
        "Login attempt started",
        Some(serde_json::json!({
            "email": request.email,
            "server_url": request.server_url
        }))
    ).await;
    
    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            let error_msg = format!("Failed to create HTTP client: {}", e);
            // Spawn async logging task
            let error_json = serde_json::json!({"error": e.to_string()});
            tokio::spawn(async move {
                crate::utils::logging::log_remote_non_blocking(
                    "login_client_error",
                    "error",
                    "Failed to create HTTP client",
                    Some(error_json)
                ).await;
            });
            error_msg
        })?;
    
    // Get device information for login
    let device_name = get_device_name();
    let platform_name = get_platform_name();
    let os_version = get_os_version();
    
    // Prepare login request with device information
    let login_url = format!("{}/api/auth/employee-login", request.server_url.trim_end_matches('/'));
    let login_data = serde_json::json!({
        "email": request.email,
        "password": request.password,
        "deviceName": device_name,
        "platform": platform_name,
        "version": os_version,
        "appVersion": env!("CARGO_PKG_VERSION")
    });

    // Make login request
    log::debug!("Sending login request to: {}", login_url);
    crate::utils::logging::log_remote_non_blocking(
        "login_request",
        "debug",
        "Sending login request",
        Some(serde_json::json!({
            "url": login_url,
            "email": request.email,
            "device_name": device_name,
            "platform": platform_name,
            "os_version": os_version
        }))
    ).await;
    
    let response = client
        .post(&login_url)
        .header("Content-Type", "application/json")
        .json(&login_data)
        .send()
        .await
        .map_err(|e| {
            let error_msg = if e.is_connect() {
                "Cannot connect to server. Please check your network connection and try again.".to_string()
            } else if e.is_timeout() {
                "Connection timeout. Please check your network connection and try again.".to_string()
            } else {
                format!("Network error: {}", e)
            };
            
            // Spawn async logging task
            let error_json = serde_json::json!({
                "error": e.to_string(),
                "error_type": if e.is_connect() { "connection" } else if e.is_timeout() { "timeout" } else { "other" }
            });
            tokio::spawn(async move {
                crate::utils::logging::log_remote_non_blocking(
                    "login_request_error",
                    "error",
                    "Login request failed",
                    Some(error_json)
                ).await;
            });
            
            error_msg
        })?;

    if response.status().is_success() {
        log::info!("Login request successful, parsing response");
        crate::utils::logging::log_remote_non_blocking(
            "login_response_success",
            "info",
            "Login request successful",
            Some(serde_json::json!({
                "status": response.status().as_u16()
            }))
        ).await;
        
        let login_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to parse response: {}", e);
                // Spawn async logging task
                let error_json = serde_json::json!({"error": e.to_string()});
                tokio::spawn(async move {
                    crate::utils::logging::log_remote_non_blocking(
                        "login_parse_error",
                        "error",
                        "Failed to parse login response",
                        Some(error_json)
                    ).await;
                });
                error_msg
            })?;

        if let Some(employee) = login_response.get("employee") {
            let employee_id = employee.get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing employee ID")?;

            // Check if device credentials are already in the login response
            let (device_id, device_token) = if let Some(device) = login_response.get("device") {
                // Handle device data from API response
                let device_id = device.get("device_id")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing device_id in device object")?;
                
                // Check if we have a token or need to handle existing token
                if let Some(device_token) = device.get("device_token").and_then(|v| v.as_str()) {
                    // New token provided
                    log::info!("Device registered with new token");
                    crate::utils::logging::log_remote_non_blocking(
                        "device_new_token",
                        "info",
                        "Device registered with new token",
                        Some(serde_json::json!({
                            "device_id": device_id,
                            "employee_id": employee_id
                        }))
                    ).await;
                    (device_id.to_string(), device_token.to_string())
                } else if device.get("token_exists").and_then(|v| v.as_bool()).unwrap_or(false) {
                    // Token exists but not provided - need to fetch it separately
                    log::info!("Device exists but token not provided, need to fetch token");
                    return Err("Device token exists but not provided. Please contact support.".to_string());
                } else {
                    return Err("Invalid device response format".to_string());
                }
            } else {
                // Device not registered, need to register it
                log::info!("Device not registered, registering new device for employee: {}", employee_id);
                crate::utils::logging::log_remote_non_blocking(
                    "device_registration_start",
                    "info",
                    "Starting device registration",
                    Some(serde_json::json!({
                        "employee_id": employee_id,
                        "device_name": device_name,
                        "platform": platform_name,
                        "os_version": os_version
                    }))
                ).await;
                
                let device_data = serde_json::json!({
                    "employeeId": employee_id,
                    "deviceName": device_name,
                    "platform": platform_name,
                    "version": os_version,
                    "appVersion": env!("CARGO_PKG_VERSION")
                });

                let register_url = format!("{}/api/devices/employee-register", request.server_url.trim_end_matches('/'));
                log::debug!("Sending device registration to: {}", register_url);
                crate::utils::logging::log_remote_non_blocking(
                    "device_registration_request",
                    "debug",
                    "Sending device registration request",
                    Some(serde_json::json!({
                        "url": register_url,
                        "employee_id": employee_id,
                        "device_name": device_name,
                        "platform": platform_name,
                        "os_version": os_version
                    }))
                ).await;
                
                let device_response = client
                    .post(&register_url)
                    .header("Content-Type", "application/json")
                    .json(&device_data)
                    .send()
                    .await
                    .map_err(|e| {
                        let error_msg = format!("Device registration error: {}", e);
                        // Spawn async logging task
                        let error_json = serde_json::json!({"error": e.to_string()});
                        tokio::spawn(async move {
                            crate::utils::logging::log_remote_non_blocking(
                                "device_registration_error",
                                "error",
                                "Device registration failed",
                                Some(error_json)
                            ).await;
                        });
                        error_msg
                    })?;

                if device_response.status().is_success() {
                    log::info!("Device registration successful");
                    crate::utils::logging::log_remote_non_blocking(
                        "device_registration_success",
                        "info",
                        "Device registration successful",
                        Some(serde_json::json!({
                            "status": device_response.status().as_u16()
                        }))
                    ).await;
                    
                    let device_result: serde_json::Value = device_response
                        .json()
                        .await
                        .map_err(|e| {
                            let error_msg = format!("Failed to parse device response: {}", e);
                            // Spawn async logging task
                            let error_json = serde_json::json!({"error": e.to_string()});
                            tokio::spawn(async move {
                                crate::utils::logging::log_remote_non_blocking(
                                    "device_registration_parse_error",
                                    "error",
                                    "Failed to parse device registration response",
                                    Some(error_json)
                                ).await;
                            });
                            error_msg
                        })?;

                    if let Some(device) = device_result.get("device") {
                        let device_id = device.get("device_id")
                            .and_then(|v| v.as_str())
                            .ok_or("Missing device_id in registration response")?;

                        let device_token = device.get("device_token")
                            .and_then(|v| v.as_str())
                            .ok_or("Missing device_token in registration response")?;
                        
                        (device_id.to_string(), device_token.to_string())
                    } else {
                        return Err("Invalid device registration response".to_string());
                    }
                } else {
                    let error_text = device_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(format!("Device registration failed: {}", error_text));
                }
            };

                    // Store credentials securely
                    {
                        let mut app_state = state.lock().await;
                        app_state.server_url = Some(request.server_url.clone());
                        app_state.device_token = Some(device_token.to_string());
                        app_state.device_id = Some(device_id.to_string());
                        app_state.email = Some(request.email.clone());
                        app_state.employee_id = Some(employee_id.to_string());
                    }

                    // Sync device token to global app state for background services
                    if let Err(e) = crate::storage::sync_device_token_to_global(
                        device_token.to_string(),
                        device_id.to_string(),
                        request.email.clone(),
                        request.server_url.clone(),
                        employee_id.to_string(),
                    ).await {
                        log::error!("Failed to sync device token to global state1: {}", e);
                    }

                    // Start background services if a work session is already active
                    if crate::storage::work_session::is_session_active().await.unwrap_or(false) {
                        log::info!("Login successful - active work session detected, starting background services");
                        let app_handle_clone = _app_handle.clone();
                        tokio::spawn(async move {
                            crate::sampling::start_all_background_services(app_handle_clone).await;
                        });
                    } else {
                        // Otherwise, services will start when the user clocks in
                        log::info!("Login successful - services will start when user clocks in");
                    }
                    crate::utils::logging::log_remote_non_blocking(
                        "login_complete",
                        "info",
                        "Login completed successfully",
                        Some(serde_json::json!({
                            "email": request.email,
                            "device_id": device_id,
                            "employee_id": employee_id
                        }))
                    ).await;

                    // Store complete session data in secure storage for persistence
                    let session_data = crate::storage::secure_store::SessionData {
                        device_token: device_token.to_string(),
                        email: request.email.clone(),
                        device_id: device_id.to_string(),
                        server_url: request.server_url.clone(),
                        employee_id: Some(employee_id.to_string()),
                    };
                    
                    if let Err(e) = crate::storage::secure_store::store_session_data(&session_data).await {
                        log::warn!("Failed to store session data securely: {}", e);
                    }
                    
                    // Also store device token separately for backward compatibility
                    if let Err(e) = crate::storage::secure_store::store_device_token(&device_token).await {
                        log::warn!("Failed to store device token securely: {}", e);
                    }

                    // Do not clear active sessions on login; respect existing clock-in state

                    // Reset app usage tracker to prevent stale sessions from causing large duration calculations
                    if let Err(e) = crate::storage::app_usage::reset_tracker().await {
                        log::warn!("Failed to reset app usage tracker: {}", e);
                    }


                    return Ok(AuthStatus {
                        is_authenticated: true,
                        email: Some(request.email),
                        device_id: Some(device_id.to_string()),
                    });
        }
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        
        // Provide more specific error messages based on status code
        let error_message = match status.as_u16() {
            401 => "Invalid email or password. Please check your credentials.",
            404 => "Server not found. Please check your network connection.",
            500 => "Server error. Please try again later.",
            _ => &error_text
        };
        
        return Err(format!("{}", error_message));
    }

    Err("Login failed".to_string())
}

#[tauri::command]
pub async fn logout(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {

    // Clear in-memory state
    {
        let mut app_state = state.lock().await;
        app_state.device_token = None;
        app_state.device_id = None;
        app_state.email = None;
        app_state.server_url = None;
        app_state.employee_id = None;
        app_state.is_paused = false;
    }

    // Also clear global app state
    if let Ok(global_state) = crate::storage::get_global_app_state() {
        let mut state = global_state.lock().await;
        state.device_token = None;
        state.device_id = None;
        state.email = None;
        state.server_url = None;
        state.employee_id = None;
        state.is_paused = false;
    }

    // Stop all background services on logout
    log::info!("Logout: Stopping all background services");
    crate::sampling::stop_services().await;

    // Reset app usage tracker to clear any active sessions
    if let Err(e) = crate::storage::app_usage::reset_tracker().await {
        log::warn!("Failed to reset app usage tracker on logout: {}", e);
    }

    // Reset idle state to prevent stale idle events
    crate::sampling::reset_idle_state();

    // Clear stored session data
    if let Err(e) = crate::storage::secure_store::delete_session_data().await {
        log::warn!("Failed to clear stored session data: {}", e);
    }
    
    // Also clear device token for backward compatibility
    if let Err(e) = crate::storage::secure_store::delete_device_token().await {
        log::warn!("Failed to clear stored device token: {}", e);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_auth_status(
    state: State<'_, Arc<Mutex<AppState>>>,
    app_handle: tauri::AppHandle,
) -> Result<AuthStatus, String> {
    let app_state = state.lock().await;
    
    // First check in-memory state
    if app_state.device_token.is_some() && app_state.email.is_some() && app_state.server_url.is_some() {
        let token = app_state.device_token.as_ref().unwrap().clone();
        let email = app_state.email.as_ref().unwrap().clone();
        let device_id = app_state.device_id.as_ref().unwrap().clone();
        let server_url = app_state.server_url.as_ref().unwrap().clone();
        
        // Validate token with server
        drop(app_state); // Release lock for async operation
        
        if let Ok(is_valid) = validate_token_with_server(&server_url, &token).await {
            if is_valid {
                // Only start services if there's an active work session
                if crate::storage::work_session::is_session_active().await.unwrap_or(false) {
                    tokio::spawn(async move {
                        crate::sampling::start_all_background_services(app_handle).await;
                    });
                }

                return Ok(AuthStatus {
                    is_authenticated: true,
                    email: Some(email),
                    device_id: Some(device_id),
                });
            } else {
                // Token is invalid, clear session
                let mut app_state = state.lock().await;
                app_state.device_token = None;
                app_state.email = None;
                app_state.device_id = None;
                app_state.server_url = None;
                app_state.employee_id = None;
                
                // Clear stored session data
                let _ = crate::storage::secure_store::delete_session_data().await;
            }
        }
    } else {
        drop(app_state); // Release lock for async operation
    }
    
    // Try to restore session from secure storage with timeout
    let restore_result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        crate::storage::secure_store::get_session_data()
    ).await;
    
    match restore_result {
        Ok(Ok(Some(session_data))) => {
            log::info!("Found stored session, validating...");
            // Validate restored token with server
            if let Ok(is_valid) = validate_token_with_server(&session_data.server_url, &session_data.device_token).await {
                if is_valid {
                    let mut app_state = state.lock().await;
                    
                    // Restore ALL session data to memory
                    app_state.device_token = Some(session_data.device_token.clone());
                    app_state.email = Some(session_data.email.clone());
                    app_state.device_id = Some(session_data.device_id.clone());
                    app_state.server_url = Some(session_data.server_url.clone());
                    app_state.employee_id = session_data.employee_id.clone();

                    // Sync device token to global app state for background services
                    if let Some(employee_id) = &session_data.employee_id {
                        if let Err(e) = crate::storage::sync_device_token_to_global(
                            session_data.device_token.clone(),
                            session_data.device_id.clone(),
                            session_data.email.clone(),
                            session_data.server_url.clone(),
                            employee_id.clone(),
                        ).await {
                            log::error!("Failed to sync device token to global state2: {}", e);
                        }

                        // Clear any existing active sessions to ensure clean state
                        if let Err(e) = crate::storage::work_session::clear_all_active_sessions().await {
                            log::warn!("Failed to clear existing active sessions: {}", e);
                        }

                        // Reset app usage tracker to prevent stale sessions from causing large duration calculations
                        if let Err(e) = crate::storage::app_usage::reset_tracker().await {
                            log::warn!("Failed to reset app usage tracker: {}", e);
                        }

                        // DO NOT automatically start background services on session restore
                        // Services should only start when user explicitly clocks in
                        log::info!("Session restored successfully - services will start when user clocks in");
                    }
                    
                    log::info!("Session restored successfully");
                    return Ok(AuthStatus {
                        is_authenticated: true,
                        email: Some(session_data.email),
                        device_id: Some(session_data.device_id),
                    });
                } else {
                    log::warn!("Stored token is invalid, clearing session");
                    // Stored token is invalid, clear it
                    let _ = crate::storage::secure_store::delete_session_data().await;
                }
            }
        }
        Ok(Ok(None)) => {
            log::info!("No stored session found");
        }
        Ok(Err(e)) => {
            log::error!("Error retrieving stored session: {}", e);
        }
        Err(_) => {
            log::error!("Timeout retrieving stored session (keychain access may be blocked)");
        }
    }
    
    // No valid session found, user needs to login
    Ok(AuthStatus {
        is_authenticated: false,
        email: None,
        device_id: None,
    })
}

// Helper function to validate token with server
async fn validate_token_with_server(server_url: &str, token: &str) -> Result<bool, String> {
    // Add timeout to prevent hanging
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let url = format!("{}/api/auth/simple-session", server_url.trim_end_matches('/'));
    
    match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let is_valid = response.status().is_success();
            if !is_valid {
                log::warn!("Token validation failed with status: {}", response.status());
            }
            Ok(is_valid)
        }
        Err(e) => {
            if e.is_connect() {
                log::warn!("Cannot connect to server for token validation: {}", e);
            } else if e.is_timeout() {
                log::warn!("Token validation timeout: {}", e);
            } else {
                log::warn!("Token validation error: {}", e);
            }
            // Return true to allow offline operation - user can still use the app
            // The actual network operations will fail gracefully and queue data
            Ok(true)
        }
    }
}

#[tauri::command]
pub async fn update_logging_config(enabled: bool, debug_mode: bool, allowed_levels: Vec<String>) -> Result<(), String> {
    crate::utils::logging::update_remote_logging_config(enabled, debug_mode, allowed_levels);
    Ok(())
}

#[tauri::command]
pub async fn get_logging_config() -> Result<serde_json::Value, String> {
    let (enabled, debug_mode, levels) = crate::utils::logging::get_remote_logging_config();
    Ok(serde_json::json!({
        "enabled": enabled,
        "debug_mode": debug_mode,
        "allowed_levels": levels
    }))
}

#[tauri::command]
pub async fn sync_logging_config() -> Result<(), String> {
    crate::utils::logging::sync_logging_config_now().await
}

#[tauri::command]
pub async fn start_logging_sync_service() -> Result<(), String> {
    crate::utils::logging::start_logging_config_sync_service().await;
    Ok(())
}

#[tauri::command]
pub async fn clear_local_database() -> Result<(), String> {
    log::info!("Clearing local database...");
    let conn = crate::storage::database::get_connection()
        .map_err(|e| format!("Failed to get database connection: {}", e))?;
    
    // Clear all tables
    conn.execute("DELETE FROM app_usage_sessions", [])
        .map_err(|e| format!("Failed to clear app_usage_sessions: {}", e))?;
    
    conn.execute("DELETE FROM work_sessions", [])
        .map_err(|e| format!("Failed to clear work_sessions: {}", e))?;
    
    conn.execute("DELETE FROM offline_queue", [])
        .map_err(|e| format!("Failed to clear offline_queue: {}", e))?;

    // Clear event and heartbeat queues to prevent residual sends
    conn.execute("DELETE FROM event_queue", [])
        .map_err(|e| format!("Failed to clear event_queue: {}", e))?;
    conn.execute("DELETE FROM heartbeat_queue", [])
        .map_err(|e| format!("Failed to clear heartbeat_queue: {}", e))?;
    
    // Reset auto-increment counters
    conn.execute("DELETE FROM sqlite_sequence WHERE name IN ('app_usage_sessions', 'work_sessions', 'offline_queue', 'event_queue', 'heartbeat_queue')", [])
        .map_err(|e| format!("Failed to reset auto-increment counters: {}", e))?;

    log::info!("Local database cleared successfully - all tables and sequences reset");
    
    Ok(())
}

#[tauri::command]
pub async fn get_recent_sessions(state: State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let (server_url, device_token, device_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone())
    };

    if let (Some(server_url), Some(device_token), Some(device_id)) = (server_url, device_token, device_id) {
        let client = reqwest::Client::new();
        
        // Call real API to get recent sessions
        let url = format!("{}/api/employees/sessions/recent", server_url);
        
        match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .header("X-Device-ID", device_id)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(sessions_data) => {
                            return Ok(sessions_data);
                        }
                        Err(e) => {
                            log::error!("Failed to parse sessions response: {}", e);
                        }
                    }
                } else {
                    log::error!("API request failed with status: {}", response.status());
                }
            }
            Err(e) => {
                log::error!("Failed to fetch sessions from API: {}", e);
            }
        }
        
        // Return empty sessions if API call fails
        return Ok(serde_json::json!({
            "sessions": []
        }));
    }

    Err("Not authenticated".to_string())
}

#[tauri::command]
pub async fn accept_consent(version: String) -> Result<(), String> {
    // Initialize database first
    if let Err(e) = crate::storage::database::init().await {
        log::error!("Failed to initialize database: {}", e);
        return Err(format!("Failed to initialize database: {}", e));
    }
    
    match consent::accept_consent(&version).await {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to accept consent: {}", e);
            Err(format!("Failed to accept consent: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_consent_status() -> Result<ConsentStatus, String> {
    // Initialize database first with timeout
    let db_init_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        crate::storage::database::init()
    ).await;
    
    match db_init_result {
        Ok(Ok(_)) => {
            log::info!("Database initialized successfully");
        }
        Ok(Err(e)) => {
            log::error!("Failed to initialize database: {}", e);
            return Err(format!("Failed to initialize database: {}", e));
        }
        Err(_) => {
            log::error!("Timeout initializing database");
            return Err("Database initialization timeout".to_string());
        }
    }
    
    // Get consent status with timeout
    let consent_result = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        consent::get_consent_status()
    ).await;
    
    match consent_result {
        Ok(Ok(status)) => Ok(ConsentStatus {
            accepted: status.accepted,
            accepted_at: status.accepted_at.map(|dt| dt.to_rfc3339()),
            version: status.version,
        }),
        Ok(Err(e)) => {
            log::error!("Failed to get consent status: {}", e);
            Err(format!("Failed to get consent status: {}", e))
        }
        Err(_) => {
            log::error!("Timeout getting consent status");
            Err("Consent status check timeout".to_string())
        }
    }
}

#[tauri::command]
pub async fn clock_in(state: State<'_, Arc<Mutex<AppState>>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    
    log::info!("Clock in: Starting clock in process");
    
    // ✅ 1. Save to LOCAL database first (this is the critical part)
    let session_id = crate::storage::work_session::start_session().await
        .map_err(|e| {
            let error_msg = format!("Failed to start local session: {}", e);
            error_msg
        })?;
    
    log::info!("Clock in: Local session started with ID {}", session_id);
    
    // ✅ 2. Start background services immediately (don't wait for backend)
    log::info!("Clock in: Starting background services");
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        crate::sampling::start_all_background_services(app_handle_clone).await;
    });
    
    // ✅ 3. Handle backend communication asynchronously (don't block clock-in)
    let (server_url, device_token, device_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone())
    };

    if let (Some(server_url), Some(device_token), Some(device_id)) = (server_url, device_token, device_id) {
        // Spawn async task to handle backend communication
        tokio::spawn(async move {
            log::info!("Clock in: Sending clock_in event to backend (async)");
            
            let client = reqwest::Client::new();
            let events_url = format!("{}/api/ingest/events", server_url.trim_end_matches('/'));
            
            let event_data = serde_json::json!({
                "events": [{
                    "type": "clock_in",
                    "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                    "data": {
                        "session_id": session_id,
                        "source": "desktop_agent"
                    }
                }]
            });

            // Try to send to backend with timeout
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                client
                    .post(&events_url)
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", device_token))
                    .header("X-Device-ID", device_id)
                    .json(&event_data)
                    .send()
            ).await {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        log::info!("Clock in: Backend event sent successfully");
                    } else {
                        log::warn!("Clock in: Backend returned error ({}), queuing event for later", response.status());
                        // Queue the clock_in event for later retry
                        if let Err(queue_err) = crate::storage::offline_queue::queue_event("clock_in", &serde_json::json!({
                            "session_id": session_id,
                            "source": "desktop_agent"
                        })).await {
                            log::error!("Failed to queue clock_in event: {}", queue_err);
                        } else {
                            log::info!("Clock in: Event queued for later delivery");
                        }
                    }
                }
                Ok(Err(e)) => {
                    // Network error, queue the event for later
                    log::warn!("Clock in: Network error, queuing event for later: {}", e);
                    
                    if let Err(queue_err) = crate::storage::offline_queue::queue_event("clock_in", &serde_json::json!({
                        "session_id": session_id,
                        "source": "desktop_agent"
                    })).await {
                        log::error!("Failed to queue clock_in event: {}", queue_err);
                    } else {
                        log::info!("Clock in: Event queued for later delivery");
                    }
                }
                Err(_) => {
                    // Timeout occurred, queue the event for later
                    log::warn!("Clock in: Backend request timeout, queuing event for later");
                    
                    if let Err(queue_err) = crate::storage::offline_queue::queue_event("clock_in", &serde_json::json!({
                        "session_id": session_id,
                        "source": "desktop_agent"
                    })).await {
                        log::error!("Failed to queue clock_in event: {}", queue_err);
                    } else {
                        log::info!("Clock in: Event queued for later delivery");
                    }
                }
            }
        });
    } else {
        return Err("Not authenticated. Please login first.".to_string());
    }
    
    Ok(())
}

#[tauri::command]
pub async fn clock_out(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    
    log::info!("Clock out: Starting clock out process");
    crate::utils::logging::log_remote_non_blocking(
        "clock_out_start",
        "info",
        "Clock out process started",
        None
    ).await;
    
    // ✅ 1. Stop background services immediately (don't wait for processing)
    crate::sampling::stop_services().await;
    log::info!("Clock out: Background services stopped");

    // ✅ 2. End LOCAL session immediately
    if let Err(e) = crate::storage::work_session::end_session().await {
        log::warn!("Clock out: Failed to end local session: {}", e);
        crate::utils::logging::log_remote_non_blocking(
            "clock_out_end_local_session_error",
            "warn",
            "Failed to end local session",
            Some(serde_json::json!({"error": e.to_string()}))
        ).await;
        // Don't fail the clock out - the session will be cleaned up later
    } else {
        log::info!("Clock out: Local session ended successfully");
        crate::utils::logging::log_remote_non_blocking(
            "clock_out_end_local_session_success",
            "info",
            "Local session ended successfully",
            None
        ).await;
    }

    // ✅ 3. Move heavy processing to background (non-blocking)
    let (server_url, device_token, device_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone())
    };

    if let (Some(server_url), Some(device_token), Some(device_id)) = (server_url, device_token, device_id) {
        // Spawn background task for heavy processing
        tokio::spawn(async move {
            log::info!("Clock out: Starting background processing");
            
            // End local app usage session
            if let Err(e) = crate::storage::app_usage::end_current_session().await {
                log::warn!("Failed to end current app session: {}", e);
            }
            
            // Send final app focus event
            if let Ok(Some(current_app)) = crate::commands::get_current_app().await {
                let event_data = serde_json::json!({
                    "app_name": current_app.name,
                    "app_id": current_app.app_id,
                    "window_title": current_app.window_title,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                if let Err(e) = crate::sampling::send_event_to_backend("app_focus", &event_data).await {
                    log::warn!("Failed to send final app focus event: {}", e);
                }
            }
            
            // Process pending events and heartbeats in background
            log::info!("Clock out: Processing remaining queued events in background");
            
            // Process pending events with timeout
            if let Ok(events) = crate::storage::offline_queue::get_pending_events().await {
                for event in events {
                    let timeout_result = tokio::time::timeout(
                        std::time::Duration::from_secs(10),
                        crate::sampling::send_event_to_backend(&event.event_type, &event.event_data)
                    ).await;
                    
                    match timeout_result {
                        Ok(Ok(_)) => {
                            let _ = crate::storage::offline_queue::mark_event_processed(event.id).await;
                            log::info!("Clock out: Processed queued event {}", event.id);
                        }
                        Ok(Err(e)) => {
                            log::warn!("Clock out: Failed to send queued event {}: {}", event.id, e);
                            let _ = crate::storage::offline_queue::mark_event_failed(event.id).await;
                        }
                        Err(_) => {
                            log::warn!("Clock out: Timeout sending queued event {}", event.id);
                            let _ = crate::storage::offline_queue::mark_event_failed(event.id).await;
                        }
                    }
                }
            }
            
            // Process pending heartbeats with timeout
            if let Ok(heartbeats) = crate::storage::offline_queue::get_pending_heartbeats().await {
                for heartbeat in heartbeats {
                    let timeout_result = tokio::time::timeout(
                        std::time::Duration::from_secs(10),
                        crate::sampling::send_heartbeat_to_backend(&heartbeat.heartbeat_data)
                    ).await;
                    
                    match timeout_result {
                        Ok(Ok(_)) => {
                            let _ = crate::storage::offline_queue::mark_heartbeat_processed(heartbeat.id).await;
                            log::info!("Clock out: Processed queued heartbeat {}", heartbeat.id);
                        }
                        Ok(Err(e)) => {
                            log::warn!("Clock out: Failed to send queued heartbeat {}: {}", heartbeat.id, e);
                            let _ = crate::storage::offline_queue::mark_heartbeat_failed(heartbeat.id).await;
                        }
                        Err(_) => {
                            log::warn!("Clock out: Timeout sending queued heartbeat {}", heartbeat.id);
                            let _ = crate::storage::offline_queue::mark_heartbeat_failed(heartbeat.id).await;
                        }
                    }
                }
            }
            
            // Send clock_out event to backend
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .connect_timeout(std::time::Duration::from_secs(5))
                .build();
                
            if let Ok(client) = client {
                let events_url = format!("{}/api/ingest/events", server_url.trim_end_matches('/'));
                
                let event_data = serde_json::json!({
                    "events": [{
                        "type": "clock_out",
                        "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                        "data": {
                            "source": "desktop_agent"
                        }
                    }]
                });

                match client
                    .post(&events_url)
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", device_token))
                    .header("X-Device-ID", device_id)
                    .json(&event_data)
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            log::info!("Clock out: Backend event sent successfully");
                        } else {
                            log::warn!("Clock out: Backend returned error ({}), queuing event for later", response.status());
                            let _ = crate::storage::offline_queue::queue_event("clock_out", &serde_json::json!({
                                "source": "desktop_agent"
                            })).await;
                        }
                    }
                    Err(e) => {
                        log::warn!("Clock out: Network error, queuing event for later: {}", e);
                        let _ = crate::storage::offline_queue::queue_event("clock_out", &serde_json::json!({
                            "source": "desktop_agent"
                        })).await;
                    }
                }
            }
            
            log::info!("Clock out: Background processing completed");
        });
    } else {
        return Err("Not authenticated. Please login first.".to_string());
    }

    // Reset idle state to prevent stale idle events
    crate::sampling::reset_idle_state();
    
    Ok(())
}

#[tauri::command]
pub async fn get_work_session(state: State<'_, Arc<Mutex<AppState>>>) -> Result<WorkSessionInfo, String> {
    // Check cache first
    {
        let app_state = state.lock().await;
        if app_state.work_session_cache.is_valid() {
            if let Some(cached_data) = app_state.work_session_cache.data.clone() {
                log::debug!("Returning cached work session data");
                return Ok(cached_data);
            }
        }
    }
    
    let (server_url, device_token, device_id, employee_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone(), app_state.employee_id.clone())
    };

    if let (Some(server_url), Some(_device_token), Some(device_id), Some(_employee_id)) = (server_url, device_token, device_id, employee_id) {
        // Fetch current work session from backend
        let client = reqwest::Client::new();
        let current_session_url = format!("{}/api/devices/current-session", server_url.trim_end_matches('/'));
       
        match client
            .get(&current_session_url)
            .header("Authorization", format!("Bearer {}", _device_token))
            .header("X-Device-ID", device_id)
            .send()
            .await {
            Ok(response) if response.status().is_success() => {
                if let Ok(session_data) = response.json::<serde_json::Value>().await {
                    let is_active = session_data.get("is_active")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    
                    if is_active {
                        // Active session found
                        let started_at = session_data.get("session")
                            .and_then(|s| s.get("clockIn"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        let current_app = session_data.get("session")
                            .and_then(|s| s.get("currentApp"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        // If no current app from backend, get it locally
                        let current_app = if current_app.is_some() {
                            current_app
                        } else {
                            match get_current_app().await {
                                Ok(Some(app)) => Some(app.name),
                                _ => None
                            }
                        };
                        
                        let session_info = WorkSessionInfo {
                            is_active: true,
                            started_at,
                            current_app,
                            idle_time_seconds: 0,
                            is_paused: false,
                        };
                        
                        // Cache the result
                        {
                            let mut app_state = state.lock().await;
                            app_state.work_session_cache.update(session_info.clone());
                        }
                        
                        return Ok(session_info);
                    } else {
                        // No active session
                        let session_info = WorkSessionInfo {
                            is_active: false,
                            started_at: None,
                            current_app: Some("TrackEx Agent".to_string()),
                            idle_time_seconds: 0,
                            is_paused: false,
                        };
                        
                        // Cache the result
                        {
                            let mut app_state = state.lock().await;
                            app_state.work_session_cache.update(session_info.clone());
                        }
                        
                        return Ok(session_info);
                    }
                }
            }
            _ => {
                // If we can't fetch from backend, fall back to local state
                log::warn!("Failed to fetch work session from backend, using local state");
                
                // Check local SQLite database for active session
                if let Ok(Some(local_session)) = crate::storage::work_session::get_current_session().await {
                    
                    // Get current app for active session
                    let current_app = match get_current_app().await {
                        Ok(Some(app)) => Some(app.name),
                        _ => None
                    };
                    
                    let session_info = WorkSessionInfo {
                        is_active: true,
                        started_at: Some(local_session.started_at.to_rfc3339()),
                        current_app,
                        idle_time_seconds: 0,
                        is_paused: false,
                    };
                    
                    // Cache the result
                    {
                        let mut app_state = state.lock().await;
                        app_state.work_session_cache.update(session_info.clone());
                    }
                    
                    return Ok(session_info);
                }
            }
        }
    }
    
    // No active session or failed to fetch
    let session_info = WorkSessionInfo {
        is_active: false,
        started_at: None,
        current_app: Some("TrackEx Agent".to_string()),
        idle_time_seconds: 0,
        is_paused: false,
    };
    
    // Cache the result
    {
        let mut app_state = state.lock().await;
        app_state.work_session_cache.update(session_info.clone());
    }
    
    Ok(session_info)
}

#[tauri::command]
pub async fn get_tracking_status(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<TrackingStatus, String> {
    let app_state = state.lock().await;
    let is_authenticated = app_state.device_token.is_some();
    
    Ok(TrackingStatus {
        is_tracking: is_authenticated,
        is_paused: app_state.is_paused,
        current_app: Some("TrackEx Agent".to_string()),
        idle_time_seconds: 0,
    })
}

#[tauri::command]
pub async fn take_screenshot() -> Result<String, String> {
    // Use the cross-platform screen capture module
    match crate::screenshots::screen_capture::capture_screen().await {
        Ok(base64_data) => {
            // Add data URI prefix if not already present
            if base64_data.starts_with("data:image/") {
                Ok(base64_data)
            } else {
                Ok(format!("data:image/jpeg;base64,{}", base64_data))
            }
        }
        Err(e) => {
            log::error!("Failed to capture screenshot: {}", e);
            Err(format!("Failed to capture screenshot: {}", e))
        }
    }
}

// Helper function to check if an app is the TrackEx Agent itself
fn is_trackex_agent(app_name: &str, app_id: &str, window_title: Option<&str>) -> bool {
    let app_name_lower = app_name.to_lowercase();
    let app_id_lower = app_id.to_lowercase();
    
    // IMPORTANT: Be very specific to avoid false positives
    // (e.g., Cursor with "trackex-desktop-agent" folder open shouldn't match)
    
    // Check app name - must be specifically "TrackEx Agent" or similar (not just containing the words)
    if app_name_lower == "trackex agent" 
        || app_name_lower == "trackex-agent" 
        || app_name_lower == "trackex_agent" {
        return true;
    }
    
    // Check app ID / bundle ID / executable name - must be the exact TrackEx executable
    if app_id_lower == "trackex-agent.exe" 
        || app_id_lower == "trackex_agent.exe"
        || app_id_lower == "trackex-agent"
        || app_id_lower == "trackex_agent"
        || app_id_lower.starts_with("com.trackex.agent")
        || app_id_lower.starts_with("com.nextup.trackex") {
        return true;
    }
    
    // Check window title ONLY if it's exactly "TrackEx Agent" or "TrackEx"
    // Do NOT check if it contains these words (to avoid false positives from folder names)
    if let Some(title) = window_title {
        let title_lower = title.trim().to_lowercase();
        if title_lower == "trackex agent" || title_lower == "trackex" {
            return true;
        }
    }
    
    false
}

#[tauri::command]
pub async fn get_current_app() -> Result<Option<AppInfo>, String> {
    // Strategy: Return the focused app, but if TrackEx is focused, return the last non-TrackEx app.
    // This ensures the UI always shows what the user is actually working on, even when viewing TrackEx.
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Primary: single AppleScript returning name and bundle id separated by ||
        let script = r#"
            tell application "System Events"
                set p to first application process whose frontmost is true
                set appName to name of p
                try
                    set bid to bundle identifier of p
                on error
                    set bid to ""
                end try
                return appName & "||" & bid
            end tell
        "#;
        if let Ok(out) = Command::new("osascript").arg("-e").arg(script).output() {
            let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
            crate::utils::logging::log_remote_non_blocking(
                "macos_app_detect_primary",
                "debug",
                "Primary AppleScript output",
                Some(serde_json::json!({"raw": raw}))
            ).await;
            if !raw.is_empty() {
                let parts: Vec<&str> = raw.split("||").collect();
                let name = parts.get(0).unwrap_or(&"").trim();
                let bundle_id = parts.get(1).unwrap_or(&"").trim();
                if !name.is_empty() {
                    let is_trackex = is_trackex_agent(name, bundle_id, None);
                    crate::utils::logging::log_remote_non_blocking(
                        "macos_app_detect_parsed",
                        "debug",
                        "Parsed macOS app",
                        Some(serde_json::json!({"name": name, "bundle_id": bundle_id, "is_trackex": is_trackex}))
                    ).await;
                    if is_trackex {
                        return Ok(crate::sampling::app_focus::get_last_non_trackex_app().await);
                    }
                    let app_info = AppInfo { name: name.to_string(), app_id: bundle_id.to_string(), window_title: Some("Active Window".to_string()) };
                    crate::sampling::app_focus::set_last_non_trackex_app(app_info.clone()).await;
                    return Ok(Some(app_info));
                }
            }
        }

        // Fallback: separate queries
        let app_name_result = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first application process whose frontmost is true")
            .output();
        let bundle_id_result = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get bundle identifier of first application process whose frontmost is true")
            .output();
        if let (Ok(name_output), Ok(bundle_output)) = (app_name_result, bundle_id_result) {
            let name = String::from_utf8_lossy(&name_output.stdout).trim().to_string();
            let bundle_id = String::from_utf8_lossy(&bundle_output.stdout).trim().to_string();
            crate::utils::logging::log_remote_non_blocking(
                "macos_app_detect_fallback",
                "debug",
                "Fallback AppleScript result",
                Some(serde_json::json!({"name": name, "bundle_id": bundle_id}))
            ).await;
            if !name.is_empty() {
                if is_trackex_agent(&name, &bundle_id, None) {
                    return Ok(crate::sampling::app_focus::get_last_non_trackex_app().await);
                }
                let app_info = AppInfo { name: name.clone(), app_id: bundle_id.clone(), window_title: Some("Active Window".to_string()) };
                crate::sampling::app_focus::set_last_non_trackex_app(app_info.clone()).await;
                return Ok(Some(app_info));
            }
        }

        // Final fallback
        crate::utils::logging::log_remote_non_blocking(
            "macos_app_detect_final_fallback",
            "warn",
            "Returning last non-TrackEx app (macOS)",
            None
        ).await;
        return Ok(crate::sampling::app_focus::get_last_non_trackex_app().await);
    }
    
    #[cfg(target_os = "windows")]
    {
        use crate::utils::windows_imports::*;
        use windows::Win32::Foundation::HWND;

        use sysinfo::{System};

        use crate::sampling::app_focus::get_windows_process_name;
        // Note: Shell API imports may not be available in this version
        // We'll use a simpler approach without UWP app detection

        unsafe {
            // Get handle to the foreground window
            let hwnd: HWND = GetForegroundWindow();
            if hwnd.0 == std::ptr::null_mut() {
                return Err("No active window".into());
            }
    
            // Get window title
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title_buf);
            let window_title = String::from_utf16_lossy(&title_buf[..len as usize]);
            let window_title = trim_nulls(&window_title);
    
            // Get process ID for identification
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            if pid == 0 {
                return Err("Failed to get process ID".into()); // Could not get process ID
            }

            // First, try to detect if this is a UWP app by checking the window
            let mut app_name = None;
            let mut app_id = None;

            if let Some(uwp_package) = crate::sampling::app_focus::get_uwp_app_from_window(hwnd) {
                app_id = Some(uwp_package.clone());
                
                // Map package family name to friendly name
                app_name = match uwp_package.as_str() {
                    "Microsoft.WindowsCalculator_8wekyb3d8bbwe" => Some("Calculator".to_string()),
                    "Microsoft.XboxGamingOverlay_8wekyb3d8bbwe" => Some("Xbox Game Bar".to_string()),
                    "Microsoft.XboxApp_8wekyb3d8bbwe" => Some("Xbox".to_string()),
                    "Microsoft.WindowsStore_8wekyb3d8bbwe" => Some("Microsoft Store".to_string()),
                    "Microsoft.Windows.Settings_8wekyb3d8bbwe" => Some("Settings".to_string()),
                    "Microsoft.Windows.ShellExperienceHost_cw5n1h2txyewy" => Some("Start Menu".to_string()),
                    _ => Some(uwp_package), // Use package name as fallback
                };
            }

            // If not UWP, use classic Win32 detection
            if app_name.is_none() {
                let mut sys = System::new_all();
                sys.refresh_all(); // Refresh process information

                if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                    let pid = process.pid().as_u32();
                    
                    // Try to get friendly name via Windows API first
                    if let Some(name) = get_windows_process_name(pid) {
                        app_name = Some(trim_nulls(&name));
                        log::debug!("Got app name from get_windows_process_name: {}", name);
                    } else {
                        // Fallback: use sysinfo to get exe path and apply mapping
                        log::debug!("get_windows_process_name returned None, using sysinfo fallback");
                        
                        // Try to get exe path from sysinfo
                        if let Some(exe_path) = process.exe() {
                            let exe_path_str = exe_path.to_string_lossy().to_string();
                            log::debug!("Process exe path: {}", exe_path_str);
                            
                            // Apply the same mapping logic
                            let exe_lower = exe_path_str.to_lowercase();
                            
                            // Check known app mappings (same as in app_focus.rs)
                            if exe_lower.contains("cursor") {
                                app_name = Some("Cursor".to_string());
                            } else if exe_lower.contains("code.exe") || (exe_lower.contains("code") && exe_lower.contains("microsoft")) {
                                app_name = Some("Visual Studio Code".to_string());
                            } else if exe_lower.contains("chrome") && !exe_lower.contains("edge") {
                                app_name = Some("Google Chrome".to_string());
                            } else if exe_lower.contains("msedge") || (exe_lower.contains("edge") && !exe_lower.contains("edgeupdate")) {
                                app_name = Some("Microsoft Edge".to_string());
                            } else if exe_lower.contains("firefox") {
                                app_name = Some("Mozilla Firefox".to_string());
                            } else if exe_lower.contains("brave") {
                                app_name = Some("Brave Browser".to_string());
                            } else if exe_lower.contains("opera") {
                                app_name = Some("Opera".to_string());
                            } else if exe_lower.contains("explorer.exe") || exe_lower.ends_with("\\explorer.exe") {
                                app_name = Some("File Explorer".to_string());
                            } else if exe_lower.contains("notepad++") {
                                app_name = Some("Notepad++".to_string());
                            } else if exe_lower.contains("notepad.exe") && !exe_lower.contains("++") {
                                app_name = Some("Notepad".to_string());
                            } else if exe_lower.contains("devenv") {
                                app_name = Some("Visual Studio".to_string());
                            } else if exe_lower.contains("teams") {
                                app_name = Some("Microsoft Teams".to_string());
                            } else if exe_lower.contains("slack") {
                                app_name = Some("Slack".to_string());
                            } else if exe_lower.contains("discord") {
                                app_name = Some("Discord".to_string());
                            } else if exe_lower.contains("zoom") {
                                app_name = Some("Zoom".to_string());
                            } else if exe_lower.contains("spotify") {
                                app_name = Some("Spotify".to_string());
                            } else if exe_lower.contains("winword") {
                                app_name = Some("Microsoft Word".to_string());
                            } else if exe_lower.contains("excel") {
                                app_name = Some("Microsoft Excel".to_string());
                            } else if exe_lower.contains("powerpnt") {
                                app_name = Some("Microsoft PowerPoint".to_string());
                            } else if exe_lower.contains("outlook") {
                                app_name = Some("Microsoft Outlook".to_string());
                            } else {
                                // Final fallback: clean filename
                                if let Some(file_name) = exe_path.file_name() {
                                    let name = file_name.to_string_lossy().to_string();
                                    // Remove .exe extension
                                    app_name = Some(if name.to_lowercase().ends_with(".exe") {
                                        name[..name.len() - 4].to_string()
                                    } else {
                                        name
                                    });
                                }
                            }
                        }
                        
                        if app_name.is_none() {
                            let proc_name = trim_nulls(process.name());
                            log::debug!("Final fallback to process.name(): {}", proc_name);
                            // Remove .exe extension if present
                            app_name = Some(if proc_name.to_lowercase().ends_with(".exe") {
                                proc_name[..proc_name.len() - 4].to_string()
                            } else {
                                proc_name
                            });
                        }
                    }
                } else {
                    log::warn!("Could not find process with PID: {}", pid);
                }
                
                // Get app ID using Windows-specific logic
                app_id = crate::sampling::app_focus::get_windows_app_id(pid);
            }
            
            let final_app_name = app_name.unwrap_or_else(|| {
                log::warn!("No app name found, using Unknown");
                "Unknown".to_string()
            });
            let final_app_id = app_id.unwrap_or_else(|| format!("pid_{}", pid));
            
            let app_info = AppInfo {
                name: final_app_name.clone(),
                app_id: final_app_id.clone(),
                window_title: Some(window_title.clone()),
            };
            
            // Check if this is the TrackEx Agent itself
            let is_trackex = is_trackex_agent(&final_app_name, &final_app_id, Some(&window_title));
            
            log::debug!("App detection: name='{}', id='{}', title='{}', is_trackex={}", 
                final_app_name, final_app_id, window_title, is_trackex);
            
            if is_trackex {
                // Return the last non-TrackEx app instead
                log::debug!("TrackEx detected as foreground, returning last non-TrackEx app");
                return Ok(crate::sampling::app_focus::get_last_non_trackex_app().await);
            }
            
            // Save this as the last non-TrackEx app
            crate::sampling::app_focus::set_last_non_trackex_app(app_info.clone()).await;
            Ok(Some(app_info))
        }
    }
    
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Fallback for other systems
        return Ok(Some(AppInfo {
            app_name: "Unknown Application".to_string(),
            app_id: "unknown".to_string(),
            window_title: Some("Unknown Window".to_string()),
        }));
    }
}

fn trim_nulls(s: &str) -> String {
    s.trim_end_matches('\u{0}').to_string()
}


#[tauri::command]
pub async fn send_diagnostics() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_permissions_status() -> Result<PermissionsStatus, String> {
    Ok(crate::permissions::get_permissions_status().await)
}

#[tauri::command]
pub async fn request_permissions() -> Result<(), String> {
    crate::permissions::request_permissions()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "name": "TrackEx Agent",
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    }))
}

#[tauri::command]
pub async fn send_app_focus_event(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let (server_url, device_token, device_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone())
    };

    if let (Some(server_url), Some(device_token), Some(device_id)) = (server_url, device_token, device_id) {
        // Get current app
        if let Ok(Some(app_info)) = get_current_app().await {
            // Send app_focus event to backend
            let client = reqwest::Client::new();
            let events_url = format!("{}/api/ingest/events", server_url.trim_end_matches('/'));
            
            let event_data = serde_json::json!({
                "events": [{
                    "type": "app_focus",
                    "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                    "data": {
                        "app_name": app_info.name,
                        "app_id": app_info.app_id,
                        "window_title": app_info.window_title.unwrap_or_default()
                    },
                    "from": "send_app_focus_event"
                }]
            });

            let response = client
                .post(&events_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", device_token))
                .header("X-Device-ID", device_id)
                .json(&event_data)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    Ok(format!("App focus tracked: {}", app_info.name))
                }
                Ok(resp) => {
                    log::error!("Failed to send app focus event: {}", resp.status());
                    Err("Failed to send app focus event".to_string())
                }
                Err(e) => {
                    log::error!("Error sending app focus event: {}", e);
                    Err("Network error sending app focus event".to_string())
                }
            }
        } else {
            Err("Could not detect current app".to_string())
        }
    } else {
        Err("Not authenticated".to_string())
    }
}

#[tauri::command]
pub async fn send_heartbeat(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let (server_url, device_token, device_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone())
    };

    if let (Some(server_url), Some(device_token), Some(device_id)) = (server_url, device_token, device_id) {
        // Get current app for heartbeat
        let current_app = match get_current_app().await {
            Ok(Some(app)) => Some(serde_json::json!({
                "name": app.name,
                "app_id": app.app_id,
                "window_title": app.window_title.unwrap_or_default()
            })),
            _ => None
        };

        // Send heartbeat to backend
        let client = reqwest::Client::new();
        let heartbeat_url = format!("{}/api/ingest/heartbeat", server_url.trim_end_matches('/'));
        
        // Get idle time and work session data for time calculations
        let idle_time = crate::sampling::idle_detector::get_idle_time().await.unwrap_or(0);
        let idle_threshold = crate::sampling::idle_detector::get_idle_threshold();
        let is_idle = idle_time >= idle_threshold;

        let now = chrono::Utc::now();

        // Check if there's an active work session for time calculations
        let session_active = crate::storage::work_session::is_session_active().await.unwrap_or(false);
        
        let (session_start, total_session_time, total_active_today, total_idle_today) = if session_active {
            // Get session start time for time calculations
            let session_start = crate::storage::work_session::get_session_start_time().await.unwrap_or_else(|_| now);
            let total_session_time = (now - session_start).num_seconds();
            
            // Calculate cumulative active and idle time for today
            let (cumulative_active_time, cumulative_idle_time) = crate::storage::work_session::get_today_time_totals().await.unwrap_or((0, 0));
            
            // Add current session time to totals
            let current_session_active = if is_idle { 0 } else { total_session_time };
            let current_session_idle = if is_idle { total_session_time } else { 0 };
            
            let total_active_today = cumulative_active_time + current_session_active;
            let total_idle_today = cumulative_idle_time + current_session_idle;


            (session_start, total_session_time, total_active_today, total_idle_today)
        } else {
            // No active session - use default values
            (now, 0, 0, 0)
        };

        let heartbeat_data = serde_json::json!({
            "timestamp": now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            "status": if is_idle { "idle" } else { "active" },
            "currentApp": current_app,
            "idle_time_seconds": idle_time,
            "session_start_time": session_start.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            "total_session_time_seconds": total_session_time,
            "active_time_today_seconds": total_active_today,
            "idle_time_today_seconds": total_idle_today,
            "is_paused": crate::sampling::is_services_paused().await
        });

        let response = client
            .post(&heartbeat_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", device_token))
            .header("X-Device-ID", device_id)
            .json(&heartbeat_data)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                Ok("Heartbeat sent".to_string())
            }
            Ok(resp) => {
                log::error!("Failed to send heartbeat2: {}", resp.status());
                Err("Failed to send heartbeat".to_string())
            }
            Err(e) => {
                log::error!("Error sending heartbeat: {}", e);
                Err("Network error sending heartbeat".to_string())
            }
        }
    } else {
        Err("Not authenticated".to_string())
    }
}

#[tauri::command]
pub async fn check_pending_jobs(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let (server_url, device_token, device_id) = {
        let app_state = state.lock().await;
        (app_state.server_url.clone(), app_state.device_token.clone(), app_state.device_id.clone())
    };

    if let (Some(server_url), Some(device_token), Some(device_id)) = (server_url, device_token, device_id) {
        let client = reqwest::Client::new();
        let jobs_url = format!("{}/api/ingest/jobs", server_url.trim_end_matches('/'));
        
        match client
            .get(&jobs_url)
            .header("Authorization", format!("Bearer {}", device_token))
            .header("X-Device-ID", device_id)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(jobs_data) = response.json::<serde_json::Value>().await {
                    if let Some(jobs) = jobs_data.get("jobs").and_then(|j| j.as_array()) {
                        for job in jobs {
                            if let Some(job_type) = job.get("type").and_then(|t| t.as_str()) {
                                if job_type == "screenshot" {
                                    if let Some(job_id) = job.get("id").and_then(|id| id.as_str()) {
                                        // Take screenshot
                                        match take_screenshot().await {
                                            Ok(screenshot_data) => {
                                                // Send screenshot completion event
                                                let event_data = serde_json::json!({
                                                    "events": [{
                                                        "type": "screenshot_taken",
                                                        "timestamp": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
                                                        "data": {
                                                            "jobId": job_id,
                                                            "job_id": job_id,
                                                            "screenshot_data": screenshot_data,
                                                            "screenshot": screenshot_data,
                                                            "auto": false
                                                        }
                                                    }]
                                                });

                                                let events_url = format!("{}/api/ingest/events", server_url.trim_end_matches('/'));
                                                let device_id = crate::storage::get_device_id().await.map_err(|_| anyhow::anyhow!("No device ID available"));
                                                let _ = client
                                                    .post(&events_url)
                                                    .header("Content-Type", "application/json")
                                                    .header("Authorization", format!("Bearer {}", device_token))
                                                    .header("X-Device-ID", device_id.expect("REASON").clone())
                                                    .json(&event_data)
                                                    .send()
                                                    .await;

                                            }
                                            Err(e) => {
                                                log::error!("Failed to take screenshot for job {}: {}", job_id, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok("Jobs checked".to_string())
            }
            Ok(response) => {
                log::error!("Failed to check jobs: {}", response.status());
                Err("Failed to check jobs".to_string())
            }
            Err(e) => {
                log::error!("Error checking jobs: {}", e);
                Err("Network error checking jobs".to_string())
            }
        }
    } else {
        Err("Not authenticated".to_string())
    }
}

#[tauri::command]
pub async fn get_idle_time() -> Result<u64, String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Use ioreg to get idle time on macOS
        let output = Command::new("ioreg")
            .arg("-c")
            .arg("IOHIDSystem")
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    let output_str = String::from_utf8_lossy(&result.stdout);
                    
                    // Parse the idle time from ioreg output
                    for line in output_str.lines() {
                        if line.contains("HIDIdleTime") {
                            if let Some(start) = line.find('=') {
                                if let Some(end) = line[start..].find(' ') {
                                    let idle_str = &line[start+1..start+end].trim();
                                    if let Ok(idle_ns) = idle_str.parse::<u64>() {
                                        // Convert nanoseconds to seconds
                                        return Ok(idle_ns / 1_000_000_000);
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(0)
            }
            Err(e) => {
                log::error!("Failed to get idle time: {}", e);
                Ok(0)
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use winapi::um::winuser::GetLastInputInfo;
        use winapi::um::winuser::LASTINPUTINFO;
        use winapi::um::sysinfoapi::GetTickCount;
    
        unsafe {
            let mut last_input_info = LASTINPUTINFO {
                cbSize: mem::size_of::<LASTINPUTINFO>() as u32,
                dwTime: 0,
            };
            
            if GetLastInputInfo(&mut last_input_info) != 0 {
                let current_time = GetTickCount();
                let idle_time_ms = current_time - last_input_info.dwTime;
                Ok(idle_time_ms as u64 / 1000) // Convert to seconds
            } else {
                Ok(0)
            }
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // For other platforms, return 0 for now
        Ok(0)
    }
}

// Removed send_idle_event command - idle detection is now handled solely by backend services
// This prevents duplicate idle events and ensures proper work session checks

#[tauri::command]
pub async fn start_background_services(app_handle: tauri::AppHandle) -> Result<(), String> {
    crate::sampling::start_all_background_services(app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn stop_background_services() -> Result<(), String> {
    crate::sampling::stop_services().await;
    Ok(())
}

#[tauri::command]
pub async fn pause_background_services() -> Result<(), String> {
    crate::sampling::pause_services().await;
    Ok(())
}

#[tauri::command]
pub async fn resume_background_services() -> Result<(), String> {
    crate::sampling::resume_services().await;
    Ok(())
}

#[tauri::command]
pub async fn get_background_service_state() -> Result<crate::sampling::BackgroundServiceState, String> {
    Ok(crate::sampling::get_service_state().await)
}

#[tauri::command]
pub async fn get_app_usage_summary() -> Result<std::collections::HashMap<String, app_usage::AppUsageSummary>, String> {
    Ok(app_usage::get_app_usage_summary().await)
}

#[tauri::command]
pub async fn get_usage_totals() -> Result<i64, String> {
    Ok(app_usage::get_usage_totals().await)
}

#[tauri::command]
pub async fn refresh_work_session(state: State<'_, Arc<Mutex<AppState>>>) -> Result<WorkSessionInfo, String> {
    // Force invalidate cache and fetch fresh data
    {
        let mut app_state = state.lock().await;
        app_state.work_session_cache.invalidate();
    }
    
    // Call get_work_session to fetch fresh data
    get_work_session(state).await
}

#[tauri::command]
pub async fn test_server_connection() -> Result<String, String> {
    match crate::storage::get_server_url().await {
        Ok(server_url) => {
            if server_url.is_empty() {
                return Err("No server URL configured".to_string());
            }
            
            // Test basic connectivity
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .connect_timeout(std::time::Duration::from_secs(3))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
            
            let test_url = format!("{}/api/health", server_url.trim_end_matches('/'));
            
            match client.get(&test_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(format!("✅ Server is reachable at {}", server_url))
                    } else {
                        Err(format!("❌ Server responded with status: {}", response.status()))
                    }
                },
                Err(e) => {
                    if e.is_connect() {
                        Err(format!("❌ Cannot connect to server at {}. Please ensure the backend is running on the correct port.", server_url))
                    } else if e.is_timeout() {
                        Err(format!("❌ Connection timeout to {}. Server may be slow or unresponsive.", server_url))
                    } else {
                        Err(format!("❌ Network error: {}", e))
                    }
                }
            }
        },
        Err(e) => Err(format!("Failed to get server URL: {}", e))
    }
}

#[tauri::command]
pub async fn get_current_app_session() -> Result<Option<app_usage::AppUsageSession>, String> {
    Ok(app_usage::get_current_session().await)
}

#[tauri::command]
pub async fn get_detailed_idle_info() -> Result<crate::sampling::idle_detector::IdleInfo, String> {
    crate::sampling::idle_detector::get_detailed_idle_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_today_report(employee_id: String, device_id: String) -> Result<crate::api::reporting::DailyReport, String> {
    crate::api::reporting::generate_today_report(employee_id, device_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_weekly_report(employee_id: String, device_id: String) -> Result<Vec<crate::api::reporting::DailyReport>, String> {
    crate::api::reporting::generate_weekly_report(employee_id, device_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_monthly_summary(employee_id: String, device_id: String) -> Result<crate::api::reporting::MonthlySummary, String> {
    crate::api::reporting::generate_monthly_summary(employee_id, device_id).await.map_err(|e| e.to_string())
}

