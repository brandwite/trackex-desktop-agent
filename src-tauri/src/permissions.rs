use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "macos")]
use core_graphics::access::ScreenCaptureAccess;

#[cfg(target_os = "windows")]
use windows::{
    Win32::{
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
    },
};

// Global flag to prevent duplicate permission requests
static PERMISSION_REQUEST_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionsStatus {
    pub screen_recording: bool,
    pub accessibility: bool,
}

impl Default for PermissionsStatus {
    fn default() -> Self {
        Self {
            screen_recording: false,
            accessibility: true, // We'll assume this is available for now
        }
    }
}

/// Check if screen recording permission is granted
pub async fn has_screen_recording_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        ScreenCaptureAccess::default().preflight()
    }
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, we can test screen capture by trying to get screen dimensions
        // If this fails, it might indicate permission issues
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            width > 0 && height > 0
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        true // Assume permission on other platforms
    }
}

/// Check if accessibility permission is granted
pub async fn has_accessibility_permission() -> bool {
    // For now, assume accessibility permission is available
    // In a real implementation, you'd check the actual permission status
    true
}

/// Get comprehensive permissions status
pub async fn get_permissions_status() -> PermissionsStatus {
    PermissionsStatus {
        screen_recording: has_screen_recording_permission().await,
        accessibility: has_accessibility_permission().await,
    }
}

/// Request permissions in a controlled, single-request manner
pub async fn request_permissions() -> Result<()> {
    // Prevent duplicate requests
    if PERMISSION_REQUEST_IN_PROGRESS.load(Ordering::Acquire) {
        return Ok(());
    }

    PERMISSION_REQUEST_IN_PROGRESS.store(true, Ordering::Release);
    
    let result = request_permissions_internal().await;
    
    PERMISSION_REQUEST_IN_PROGRESS.store(false, Ordering::Release);
    
    result
}

async fn request_permissions_internal() -> Result<()> {
    
    #[cfg(target_os = "macos")]
    {
        log::info!("Requesting screen recording permission...");
        
        // Request screen recording permission
        if !has_screen_recording_permission().await {
            log::info!("Screen recording permission not granted, requesting...");
            
            // The request() method triggers the permission dialog
            let result = ScreenCaptureAccess::default().request();
            log::info!("Screen recording permission request result: {:?}", result);
            
            // Small delay to allow dialog to appear
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        } else {
            log::info!("Screen recording permission already granted");
        }
        
        // Give macOS time to show permission dialogs and user to respond
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, most permissions are granted by default
        // We can test screen capture capability
        
        // Try to get screen dimensions as a test
        let has_screen_access = has_screen_recording_permission().await;
        if has_screen_access {
        } else {
            log::warn!("Windows screen capture access may be limited");
        }
        
        // Windows doesn't require explicit permission requests for most functionality
        // The app will work with the permissions it has
    }
    
    Ok(())
}

/// Open macOS Privacy Settings to the Screen Recording section
#[allow(dead_code)]
pub async fn open_privacy_settings() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()?;
    }
    
    Ok(())
}

/// Check if all required permissions are granted
#[allow(dead_code)]
pub async fn are_required_permissions_granted() -> bool {
    let status = get_permissions_status().await;
    status.screen_recording
}
