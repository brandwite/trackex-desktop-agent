use anyhow::Result;

#[cfg(target_os = "macos")]
use core_graphics::access::ScreenCaptureAccess;

#[allow(dead_code)]
pub async fn has_screen_recording_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        ScreenCaptureAccess::default().preflight()
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        true // Assume permission on other platforms
    }
}

#[allow(dead_code)]
pub async fn request_screen_recording_permission() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // On macOS, we can't programmatically request screen recording permission
        // The user must grant it in System Preferences
        
        // Attempt to trigger the permission dialog by trying to access screen content
        let _ = ScreenCaptureAccess::default().request();
    }
    
    Ok(())
}

