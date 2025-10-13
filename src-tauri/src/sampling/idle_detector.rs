use anyhow::Result;

// Unused imports removed for macOS - kept for future reference if needed
// #[cfg(target_os = "macos")]
// use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

#[cfg(target_os = "windows")]
use winapi::{
    um::winuser::{GetLastInputInfo, LASTINPUTINFO},
    um::sysinfoapi::GetTickCount,
};

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub async fn get_idle_time() -> Result<u64> {
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
#[allow(dead_code)]
pub async fn get_idle_time() -> Result<u64> {
    use std::mem;
    
    unsafe {
        let mut last_input_info = LASTINPUTINFO {
            cbSize: mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        
        if GetLastInputInfo(&mut last_input_info) != 0 {
            let current_time = GetTickCount();
            let idle_time_ms = current_time - last_input_info.dwTime;
            return Ok(idle_time_ms as u64 / 1000) // Convert to seconds
        } else {
            return Ok(0)
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub async fn get_system_idle_time() -> Result<u64> {
    // Use the existing get_idle_time function
    get_idle_time().await
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub async fn is_system_idle(threshold_seconds: u64) -> Result<bool> {
    let idle_time = get_idle_time().await?;
    Ok(idle_time >= threshold_seconds)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub async fn get_system_idle_time() -> Result<u64> {
    // Use the existing get_idle_time function
    get_idle_time().await
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub async fn is_system_idle(threshold_seconds: u64) -> Result<bool> {
    let idle_time = get_idle_time().await?;
    Ok(idle_time >= threshold_seconds)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
#[allow(dead_code)]
pub async fn get_detailed_idle_info() -> Result<IdleInfo> {
    let idle_time = get_idle_time().await?;
    let threshold = get_idle_threshold();
    let is_idle = idle_time >= threshold;
    
    Ok(IdleInfo {
        idle_time_seconds: idle_time,
        threshold_seconds: threshold,
        is_idle,
        last_activity_time: chrono::Utc::now() - chrono::Duration::seconds(idle_time as i64),
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdleInfo {
    pub idle_time_seconds: u64,
    pub threshold_seconds: u64,
    pub is_idle: bool,
    pub last_activity_time: chrono::DateTime<chrono::Utc>,
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub async fn get_idle_time() -> Result<u64> {
    // Placeholder for other platforms
    Ok(0)
}

#[allow(dead_code)]
pub async fn is_user_idle(threshold_seconds: u64) -> Result<bool> {
    let idle_time = get_idle_time().await?;
    Ok(idle_time >= threshold_seconds)
}

#[allow(dead_code)]
pub fn get_idle_threshold() -> u64 {
    // Default idle threshold: 5 minutes (300 seconds)
    std::env::var("TRACKEX_IDLE_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120)
}
