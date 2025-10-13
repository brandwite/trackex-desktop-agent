use anyhow::Result;

pub async fn has_screen_recording_permission() -> bool {
    crate::screenshots::permissions::has_screen_recording_permission().await
}

pub async fn has_accessibility_permission() -> bool {
    // For now, assume accessibility permission is available
    // In a real implementation, you'd check the actual permission status
    true
}

pub async fn request_permissions() {
    
    if !has_screen_recording_permission().await {
        if let Err(e) = crate::screenshots::permissions::request_screen_recording_permission().await {
            log::error!("Failed to request screen recording permission: {}", e);
        }
    }
    
    // For accessibility permission, we'd typically show a dialog directing
    // the user to System Preferences > Security & Privacy > Accessibility
}

pub async fn open_privacy_settings() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn()?;
    }
    
    Ok(())
}

pub async fn get_permissions_status() -> (bool, bool) {
    let screen_recording = has_screen_recording_permission().await;
    let accessibility = has_accessibility_permission().await;
    
    (screen_recording, accessibility)
}

