use anyhow::Result;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
const SERVICE_NAME: &str = "com.trackex.agent";
#[allow(dead_code)]
const DEVICE_TOKEN_KEY: &str = "device_token";
#[allow(dead_code)]
const SESSION_DATA_KEY: &str = "session_data";

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub device_token: String,
    pub email: String,
    pub device_id: String,
    pub server_url: String,
    pub employee_id: Option<String>,
}

pub async fn store_device_token(token: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        
        // Use a consistent service and account name 
        let entry = Entry::new(SERVICE_NAME, DEVICE_TOKEN_KEY)?;
        
        // Store directly without checking existing - this reduces keychain prompts
        entry.set_password(token)?;
    }
    
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincred::*;
        use std::ffi::CString;
        use std::ptr;
        
        unsafe {
            let target_name = CString::new(format!("{}:{}", SERVICE_NAME, DEVICE_TOKEN_KEY))?;
            let credential_blob = token.as_bytes();
            
            let mut credential = CREDENTIALW {
                Flags: 0,
                Type: CRED_TYPE_GENERIC,
                TargetName: target_name.as_ptr() as *mut u16,
                Comment: ptr::null_mut(),
                LastWritten: winapi::shared::minwindef::FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 },
                CredentialBlobSize: credential_blob.len() as u32,
                CredentialBlob: credential_blob.as_ptr() as *mut u8,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: ptr::null_mut(),
                TargetAlias: ptr::null_mut(),
                UserName: ptr::null_mut(),
            };
            
            if CredWriteW(&mut credential, 0) != 0 {
            } else {
                log::error!("Failed to store device token in Windows Credential Manager");
                return Err(anyhow::anyhow!("Failed to store device token"));
            }
        }
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        log::warn!("Secure storage not implemented for this platform");
    }
    
    Ok(())
}

#[allow(dead_code)]
pub async fn get_device_token() -> Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        let entry = Entry::new(SERVICE_NAME, DEVICE_TOKEN_KEY)?;
        match entry.get_password() {
            Ok(token) => {
                return Ok(Some(token));
            }
            Err(keyring::Error::NoEntry) => {
                return Ok(None);
            }
            Err(e) => {
                log::error!("Failed to retrieve device token: {}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Secure storage not implemented for this platform");
        Ok(None)
    }
}

pub async fn delete_device_token() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        let entry = Entry::new(SERVICE_NAME, DEVICE_TOKEN_KEY)?;
        match entry.delete_password() {
            Ok(_) => {
            }
            Err(keyring::Error::NoEntry) => {
            }
            Err(e) => {
                log::error!("Failed to delete device token: {}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Secure storage not implemented for this platform");
    }
    
    Ok(())
}

pub async fn store_session_data(_session: &SessionData) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        
        let entry = Entry::new(SERVICE_NAME, SESSION_DATA_KEY)?;
        let session_json = serde_json::to_string(_session)?;
        entry.set_password(&session_json)?;
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Secure storage not implemented for this platform");
    }
    
    Ok(())
}

pub async fn get_session_data() -> Result<Option<SessionData>> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        log::info!("Attempting to retrieve session data from keychain...");
        
        match Entry::new(SERVICE_NAME, SESSION_DATA_KEY) {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(session_json) => {
                        log::info!("Session data retrieved from keychain");
                        match serde_json::from_str::<SessionData>(&session_json) {
                            Ok(session) => {
                                return Ok(Some(session));
                            }
                            Err(e) => {
                                log::error!("Failed to parse session data: {}", e);
                                return Err(e.into());
                            }
                        }
                    }
                    Err(keyring::Error::NoEntry) => {
                        log::info!("No session data found in keychain");
                        return Ok(None);
                    }
                    Err(e) => {
                        log::error!("Failed to retrieve session data from keychain: {}", e);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create keychain entry: {}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Secure storage not implemented for this platform");
        Ok(None)
    }
}

pub async fn delete_session_data() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        let entry = Entry::new(SERVICE_NAME, SESSION_DATA_KEY)?;
        match entry.delete_password() {
            Ok(_) => {
            }
            Err(keyring::Error::NoEntry) => {
            }
            Err(e) => {
                log::error!("Failed to delete session data: {}", e);
                return Err(e.into());
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Secure storage not implemented for this platform");
    }
    
    Ok(())
}

#[allow(dead_code)]
pub async fn get_server_url() -> Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        use keyring::Entry;
        let entry = Entry::new(SERVICE_NAME, "server_url")?;
        match entry.get_password() {
            Ok(url) => {
                return Ok(Some(url));
            }
            Err(_) => {
                return Ok(None);
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Secure storage not implemented for this platform");
        Ok(None)
    }
}