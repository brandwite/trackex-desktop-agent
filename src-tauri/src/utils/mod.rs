pub mod logging;
pub mod productivity;

#[cfg(target_os = "windows")]
pub mod windows_imports {
    pub use std::ffi::OsString;
    pub use std::os::windows::ffi::OsStringExt;
    pub use std::os::windows::ffi::OsStrExt;
    pub use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    pub use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow,
        GetWindowTextW,
        GetWindowThreadProcessId,
        GetWindow,
        IsWindowVisible,
        GW_HWNDNEXT,
    };
    pub use winapi::um::winver::{
        GetFileVersionInfoSizeW,
        GetFileVersionInfoW,
        VerQueryValueW,
    };
    pub use winapi::um::processthreadsapi::OpenProcess;
    pub use winapi::um::winnt::{
        PROCESS_QUERY_INFORMATION,
        PROCESS_VM_READ,
    };    
}