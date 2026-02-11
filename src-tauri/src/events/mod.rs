pub mod system_events;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

pub use system_events::{auto_pause_if_running, AutoPauseEvent, AutoPauseReason};

use crate::db::DbConnection;
use tauri::AppHandle;

/// Initialize platform-specific system event listeners
pub fn initialize_system_listeners(app_handle: AppHandle, db: DbConnection) {
    #[cfg(target_os = "windows")]
    {
        windows::initialize_windows_listener(app_handle, db);
    }

    #[cfg(target_os = "macos")]
    {
        macos::initialize_macos_listener(app_handle, db);
    }

    #[cfg(target_os = "linux")]
    {
        linux::initialize_linux_listener(app_handle, db);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        eprintln!("Warning: System event listeners not available for this platform");
        let _ = (app_handle, db); // Prevent unused variable warnings
    }
}
