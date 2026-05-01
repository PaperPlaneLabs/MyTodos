use crate::db::DbConnection;
use crate::error::Result;
use crate::services::window_tracking_service::{
    self, WindowActivityStats, WindowTrackingSettings, WindowTrackingState,
};
use tauri::State;

#[tauri::command]
pub fn get_window_tracking_settings(db: State<DbConnection>) -> Result<WindowTrackingSettings> {
    window_tracking_service::get_settings(db.inner())
}

#[tauri::command]
pub fn set_window_tracking_enabled(
    db: State<DbConnection>,
    enabled: bool,
) -> Result<WindowTrackingSettings> {
    window_tracking_service::set_enabled(db.inner(), enabled)
}

#[tauri::command]
pub fn get_window_tracking_state(db: State<DbConnection>) -> Result<WindowTrackingState> {
    window_tracking_service::get_state(db.inner())
}

#[tauri::command]
pub fn set_window_tracking_paused(
    db: State<DbConnection>,
    paused: bool,
) -> Result<WindowTrackingState> {
    window_tracking_service::set_paused(db.inner(), paused)
}

#[tauri::command]
pub fn get_window_activity_stats(db: State<DbConnection>) -> Result<WindowActivityStats> {
    window_tracking_service::get_stats(db.inner())
}

#[tauri::command]
pub fn clear_window_activity(db: State<DbConnection>) -> Result<()> {
    window_tracking_service::clear_activity(db.inner())
}
