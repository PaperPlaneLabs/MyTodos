use crate::db::DbConnection;
use crate::events;
use std::sync::atomic::Ordering;
use tauri::{App, AppHandle, Manager, Wry};

#[cfg(target_os = "windows")]
fn is_os_shutting_down() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SHUTTINGDOWN};
    unsafe { GetSystemMetrics(SM_SHUTTINGDOWN) != 0 }
}

#[cfg(not(target_os = "windows"))]
fn is_os_shutting_down() -> bool {
    false
}

pub fn register_main_window_close_behavior(
    app: &App<Wry>,
    app_handle: AppHandle,
    db: DbConnection,
) {
    if let Some(window) = app.get_webview_window("main") {
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let is_os_shutting_down = is_os_shutting_down();

                if !events::is_shutting_down() && !is_os_shutting_down {
                    api.prevent_close();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.hide();
                    }
                } else {
                    println!("Shutdown or App Quit detected, pausing timers...");
                    events::SHUTTING_DOWN.store(true, Ordering::SeqCst);
                    events::auto_pause_if_running(
                        &app_handle,
                        &db,
                        events::AutoPauseReason::Shutdown,
                    );
                    if let Err(error) =
                        crate::services::window_tracking_service::pause_tracking(&db)
                    {
                        eprintln!("Failed to pause window tracking during shutdown: {}", error);
                    }
                }
            }
        });
    }
}
