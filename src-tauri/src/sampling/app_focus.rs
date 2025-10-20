use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::OnceLock;

#[cfg(not(target_os = "windows"))]
use anyhow::Result;

// use crate::storage::app_usage;
// use crate::api::app_rules;

use crate::commands::get_current_app;
use crate::storage::app_usage;
use crate::sampling::idle_detector;

// Global state to track the last non-TrackEx app
static LAST_NON_TRACKEX_APP: OnceLock<Arc<Mutex<Option<AppInfo>>>> = OnceLock::new();

fn get_last_app_state() -> &'static Arc<Mutex<Option<AppInfo>>> {
    LAST_NON_TRACKEX_APP.get_or_init(|| Arc::new(Mutex::new(None)))
}

pub async fn set_last_non_trackex_app(app: AppInfo) {
    let state = get_last_app_state();
    let mut last_app = state.lock().await;
    *last_app = Some(app);
}

pub async fn get_last_non_trackex_app() -> Option<AppInfo> {
    let state = get_last_app_state();
    let last_app = state.lock().await;
    last_app.clone()
}

// Unused imports removed for macOS - kept for future reference if needed
// #[cfg(target_os = "macos")]
// use cocoa::{
//     base::{id, nil},
//     foundation::{NSString, NSAutoreleasePool},
// };
//
// #[cfg(target_os = "macos")]
// use core_foundation::string::CFString;
//
// #[cfg(target_os = "macos")]
// use objc::{msg_send, sel, sel_impl};

// #[cfg(target_os = "windows")]
// use windows::{
//     Win32::{
//         Foundation::CloseHandle,
//         System::Diagnostics::ToolHelp::{
//             CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
//             TH32CS_SNAPPROCESS,
//         },
//     },
// };
#[cfg(target_os = "windows")]
use crate::utils::windows_imports::*;

// #[cfg(target_os = "windows")]
// use winapi::um::handleapi::CloseHandle;

use crate::utils::productivity::ProductivityClassifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub app_id: String,
    pub window_title: Option<String>,
}

#[allow(dead_code)]
pub async fn start_sampling(_app_handle: AppHandle) {
    let interval_seconds = super::get_app_focus_interval();

    // Initialize app usage tracker and productivity classifier
    let classifier = ProductivityClassifier::with_default_rules();
    
    // Wait a bit for database initialization to complete
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
    let mut last_app_info: Option<crate::sampling::app_focus::AppInfo> = None;
    
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

        if let Ok(app_info_opt) = get_current_app().await {
                if let Some(app_info) = app_info_opt {
                    // Check if app has changed
                    let app_changed = last_app_info.as_ref().map_or(true, |last| {
                        last.name != app_info.name || last.app_id != app_info.app_id
                    });
                    
                    // Get idle status
                    let idle_time = idle_detector::get_idle_time().await.unwrap_or(0);
                    let idle_threshold = idle_detector::get_idle_threshold();
                    let is_idle = idle_time >= idle_threshold;
                    
                    if app_changed {
                        log::info!("📱 App focus changed: {} ({})", app_info.name, app_info.app_id);
                        
                        // Trigger immediate heartbeat to reflect app change in real-time
                        super::heartbeat::trigger_immediate_heartbeat().await;
                        // Remote debug log
                        crate::utils::logging::log_remote_non_blocking(
                            "app_focus_change",
                            "info",
                            "Detected app change",
                            Some(serde_json::json!({
                                "name": app_info.name,
                                "app_id": app_info.app_id,
                                "window_title": app_info.window_title,
                            }))
                        ).await;
                        
                        // End previous session if it exists
                        if let Err(e) = app_usage::end_current_session().await {
                            log::warn!("Failed to end current app session: {}", e);
                        }
                        
                        // Classify the new app
                        let category = classifier.classify_app(
                            &app_info.name, 
                            &app_info.app_id, 
                            app_info.window_title.as_deref()
                        );
                        
                        log::debug!("App classified as: {}", category);
                        
                        // Start new session
                        if let Err(e) = app_usage::start_app_session(
                            app_info.name.clone(),
                            app_info.app_id.clone(),
                            app_info.window_title.clone(),
                            category.clone(),
                            is_idle,
                        ).await {
                            log::error!("Failed to start new app session: {}", e);
                        }
                        
                        // Send app focus event ONLY when app changes
                        let event_data = serde_json::json!({
                            "app_name": app_info.name,
                            "app_id": app_info.app_id,
                            "window_title": app_info.window_title,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        });

                        // Try to send immediately for real-time updates
                        match crate::sampling::send_event_to_backend("app_focus", &event_data).await {
                            Ok(_) => {
                                log::info!("✓ App focus event sent: {}", app_info.name);
                                crate::utils::logging::log_remote_non_blocking(
                                    "app_focus_sent",
                                    "info",
                                    "App focus event sent",
                                    Some(event_data.clone())
                                ).await;
                            }
                            Err(e) => {
                                // Only queue if immediate send fails (network issue, etc)
                                log::warn!("Failed to send app focus event live, queuing: {}", e);
                                if let Err(queue_err) = crate::storage::offline_queue::queue_event("app_focus", &event_data).await {
                                    log::error!("CRITICAL: Failed to queue app focus event: {}", queue_err);
                                    crate::utils::logging::log_remote_non_blocking(
                                        "app_focus_queue_failed",
                                        "error",
                                        &format!("{}", queue_err),
                                        Some(event_data.clone())
                                    ).await;
                                } else {
                                    log::debug!("App focus event queued for later delivery");
                                    crate::utils::logging::log_remote_non_blocking(
                                        "app_focus_queued",
                                        "warn",
                                        "App focus queued for later delivery",
                                        Some(event_data.clone())
                                    ).await;
                                }
                            }
                        }
                        
                        last_app_info = Some(app_info.clone());
                    } else {
                        // App hasn't changed, just update current session's idle status
                        if let Err(e) = app_usage::update_current_session(is_idle).await {
                            log::warn!("Failed to update session idle status: {}", e);
                        }
                    }
                } else {
                    log::trace!("No app detected in current check");
                }
        } else {
            log::trace!("Failed to get current app");
        }

        interval.tick().await;
    }

    // End the last session when stopping
    if let Err(e) = app_usage::end_current_session().await {
        log::warn!("Failed to end final app session: {}", e);
    }

}

// #[cfg(target_os = "macos")]
// pub async fn get_current_app() -> Result<AppInfo> {
//     use std::process::Command;
    
//     // Get the frontmost application using AppleScript
//     let app_name_result = Command::new("osascript")
//         .arg("-e")
//         .arg("tell application \"System Events\" to get name of first application process whose frontmost is true")
//         .output();
        
//     let bundle_id_result = Command::new("osascript")
//         .arg("-e")
//         .arg("tell application \"System Events\" to get bundle identifier of first application process whose frontmost is true")
//         .output();
        
//     match (app_name_result, bundle_id_result) {
//         (Ok(name_output), Ok(bundle_output)) => {
//             let name = String::from_utf8_lossy(&name_output.stdout).trim().to_string();
//             let bundle_id = String::from_utf8_lossy(&bundle_output.stdout).trim().to_string();
            
//             if !name.is_empty() {
//                 // Try to get window title (may fail due to permissions)
//                 let window_title = get_window_title().await.ok();
                
//                 Ok(AppInfo {
//                     name: name.to_string(),
//                     app_id: if bundle_id.is_empty() { "unknown.bundle.id".to_string() } else { bundle_id.to_string() },
//                     window_title,
//                 })
//             } else {
//                 Ok(AppInfo {
//                     name: "Unknown Application".to_string(),
//                     app_id: "unknown.bundle.id".to_string(),
//                     window_title: None,
//                 })
//             }
//         }
//         _ => {
//             log::debug!("Failed to get current app via AppleScript");
//             Ok(AppInfo {
//                 name: "Unknown Application".to_string(),
//                 app_id: "unknown.bundle.id".to_string(),
//                 window_title: None,
//             })
//         }
//     }
// }

#[cfg(target_os = "macos")]
async fn get_window_title() -> Result<String> {
    // This is a simplified implementation
    // In a real app, you'd use the Accessibility API to get the window title
    // For now, we'll return None as window titles require additional permissions
    Err(anyhow::anyhow!("Window title access not implemented"))
}

#[cfg(target_os = "windows")]
pub fn get_windows_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            0, // FALSE
            pid,
        );
        
        if handle.is_null() {
            return None;
        }

        // Get the exe path
        let mut buffer = vec![0u16; 260];
        let windows_handle = windows::Win32::Foundation::HANDLE(handle as *mut std::ffi::c_void);
        let len = GetModuleFileNameExW(Some(windows_handle), None, &mut buffer);
        if len == 0 {
            return None;
        }
        buffer.truncate(len as usize);
        let exe_path = OsString::from_wide(&buffer);

        // Try multiple methods to get the app name
        if let Some(name) = get_app_name_from_version_info(&exe_path) {
            return Some(name);
        }

        if let Some(name) = get_app_name_from_shell(&exe_path) {
            return Some(name);
        }

        if let Some(name) = get_app_name_from_mapping(&exe_path) {
            return Some(name);
        }

        // Final fallback: clean filename
        get_clean_filename(&exe_path)
    }
}

#[cfg(target_os = "windows")]
pub fn get_windows_app_id(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            0, // FALSE
            pid,
        );
        
        if handle.is_null() {
            return None;
        }

        // First, try to detect if this is a UWP app
        let windows_handle = windows::Win32::Foundation::HANDLE(handle as *mut std::ffi::c_void);
        if let Some(package_family_name) = get_uwp_package_family_name(windows_handle) {
            return Some(package_family_name);
        }

        // For classic Win32 apps, get the executable name
        let mut buffer = vec![0u16; 260];
        let len = GetModuleFileNameExW(Some(windows_handle), None, &mut buffer);
        if len == 0 {
            return None;
        }
        buffer.truncate(len as usize);
        let exe_path = OsString::from_wide(&buffer);

        // Extract just the executable name (e.g., "chrome.exe")
        if let Some(path_str) = exe_path.to_str() {
            if let Some(filename) = std::path::Path::new(path_str).file_name() {
                if let Some(name_str) = filename.to_str() {
                    return Some(name_str.to_string());
                }
            }
        }

        None
    }
}

#[cfg(target_os = "windows")]
fn get_uwp_package_family_name(handle: windows::Win32::Foundation::HANDLE) -> Option<String> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER};
    use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        // first call to get required buffer length
        let mut length: u32 = 0;
        let hr = GetPackageFamilyName(handle, &mut length, None);
        // win32 success is 0 (ERROR_SUCCESS); insufficient buffer is ERROR_INSUFFICIENT_BUFFER
        if hr.0 != 0 && hr.0 != ERROR_INSUFFICIENT_BUFFER.0 {
            return None;
        }
        if length == 0 {
            return None;
        }

        // allocate buffer (length includes null terminator)
        let mut buf: Vec<u16> = vec![0u16; length as usize];

        // create PWSTR pointing at our buffer
        let pw = PWSTR::from_raw(buf.as_mut_ptr());

        let hr = GetPackageFamilyName(handle, &mut length, Some(pw));
        if hr.0 != 0 {
            return None;
        }

        // length is number of chars INCLUDING null terminator
        if length == 0 {
            return None;
        }
        // drop trailing null and convert
        let str_len = (length - 1) as usize;
        buf.truncate(str_len);
        let s = OsString::from_wide(&buf).to_string_lossy().into_owned();
        Some(s)
    }
}

#[cfg(target_os = "windows")]
pub fn get_uwp_app_from_window(hwnd: windows::Win32::Foundation::HWND) -> Option<String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId};
    
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        
        if pid == 0 {
            return None;
        }
        
        // Try to get UWP package info from process
        let handle = winapi::um::processthreadsapi::OpenProcess(
            winapi::um::winnt::PROCESS_QUERY_INFORMATION,
            0,
            pid,
        );
        
        if handle.is_null() {
            return None;
        }
        
        let windows_handle = windows::Win32::Foundation::HANDLE(handle as *mut std::ffi::c_void);
        let package_name = get_uwp_package_family_name(windows_handle);
        
        let _ = winapi::um::handleapi::CloseHandle(handle);
        package_name
    }
}


#[cfg(target_os = "windows")]
fn get_app_name_from_version_info(exe_path: &OsString) -> Option<String> {
    unsafe {
        let exe_path_wide: Vec<u16> = exe_path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let size = GetFileVersionInfoSizeW(exe_path_wide.as_ptr(), std::ptr::null_mut());
        if size == 0 {
            return None;
        }

        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(exe_path_wide.as_ptr(), 0, size, data.as_mut_ptr() as *mut _) == 0 {
            return None;
        }

        let mut ptr: *mut winapi::ctypes::c_void = std::ptr::null_mut();
        let mut len: u32 = 0;
        let sub_block: Vec<u16> = "\\StringFileInfo\\040904b0\\FileDescription\0".encode_utf16().collect();
        if VerQueryValueW(
            data.as_mut_ptr() as *mut _,
            sub_block.as_ptr() as *mut u16,
            &mut ptr,
            &mut len,
        ) != 0 {
            let slice = std::slice::from_raw_parts(ptr as *const u16, len as usize);
            let friendly_name = OsString::from_wide(slice);
            if let Some(name) = friendly_name.to_str() {
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }
}

#[cfg(target_os = "windows")]
fn get_app_name_from_shell(_exe_path: &OsString) -> Option<String> {
    // This would use Shell32 API to get file description
    // For now, return None as we need to add Shell32 imports
    None
}

#[cfg(target_os = "windows")]
fn get_app_name_from_mapping(exe_path: &OsString) -> Option<String> {
    if let Some(path_str) = exe_path.to_str() {
        let path_lower = path_str.to_lowercase();
        
        // Known app mappings - order matters, check specific apps first
        // Cursor (check before generic 'code' check)
        if path_lower.contains("cursor") {
            return Some("Cursor".to_string());
        }
        // VS Code (be specific)
        if path_lower.contains("code.exe") || (path_lower.contains("code") && path_lower.contains("microsoft")) {
            return Some("Visual Studio Code".to_string());
        }
        // Browsers
        if path_lower.contains("firefox") {
            return Some("Mozilla Firefox".to_string());
        }
        if path_lower.contains("chrome") && !path_lower.contains("edge") {
            return Some("Google Chrome".to_string());
        }
        if path_lower.contains("msedge") || path_lower.contains("edge") {
            return Some("Microsoft Edge".to_string());
        }
        if path_lower.contains("brave") {
            return Some("Brave Browser".to_string());
        }
        if path_lower.contains("opera") {
            return Some("Opera".to_string());
        }
        // System apps
        if path_lower.contains("notepad++") {
            return Some("Notepad++".to_string());
        }
        if path_lower.contains("notepad") && !path_lower.contains("++") {
            return Some("Notepad".to_string());
        }
        if path_lower.contains("applicationframehost") {
            return None; // Let UWP detection handle this
        }
        // IDEs and editors
        if path_lower.contains("devenv") {
            return Some("Visual Studio".to_string());
        }
        if path_lower.contains("pycharm") {
            return Some("PyCharm".to_string());
        }
        if path_lower.contains("idea") && (path_lower.contains("jetbrains") || path_lower.contains("intellij")) {
            return Some("IntelliJ IDEA".to_string());
        }
        if path_lower.contains("webstorm") {
            return Some("WebStorm".to_string());
        }
        if path_lower.contains("sublime") {
            return Some("Sublime Text".to_string());
        }
        if path_lower.contains("atom") {
            return Some("Atom".to_string());
        }
        // File managers
        if path_lower.contains("explorer.exe") || path_lower.contains("explorer") {
            return Some("File Explorer".to_string());
        }
        // Microsoft Office
        if path_lower.contains("winword") {
            return Some("Microsoft Word".to_string());
        }
        if path_lower.contains("excel") {
            return Some("Microsoft Excel".to_string());
        }
        if path_lower.contains("powerpnt") {
            return Some("Microsoft PowerPoint".to_string());
        }
        if path_lower.contains("outlook") {
            return Some("Microsoft Outlook".to_string());
        }
        // Communication
        if path_lower.contains("teams") {
            return Some("Microsoft Teams".to_string());
        }
        if path_lower.contains("slack") {
            return Some("Slack".to_string());
        }
        if path_lower.contains("discord") {
            return Some("Discord".to_string());
        }
        if path_lower.contains("zoom") {
            return Some("Zoom".to_string());
        }
        // Media
        if path_lower.contains("spotify") {
            return Some("Spotify".to_string());
        }
        if path_lower.contains("vlc") {
            return Some("VLC Media Player".to_string());
        }
        // Adobe
        if path_lower.contains("photoshop") {
            return Some("Adobe Photoshop".to_string());
        }
        if path_lower.contains("illustrator") {
            return Some("Adobe Illustrator".to_string());
        }
        if path_lower.contains("acrobat") {
            return Some("Adobe Acrobat".to_string());
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_clean_filename(exe_path: &OsString) -> Option<String> {
    if let Some(path_str) = exe_path.to_str() {
        if let Some(filename) = std::path::Path::new(path_str).file_name() {
            if let Some(name_str) = filename.to_str() {
                let name = name_str.to_string();
                
                // Remove .exe extension if present
                if name.to_lowercase().ends_with(".exe") {
                    return Some(name[..name.len() - 4].to_string());
                }
                return Some(name);
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn get_current_app() -> Result<AppInfo> {
    // Placeholder for other platforms
    Ok(AppInfo {
        name: "Unknown".to_string(),
        app_id: "unknown.bundle.id".to_string(),
        window_title: None,
    })
}
