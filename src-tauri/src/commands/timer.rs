use crate::db::{ActiveTimer, DbConnection, TimeEntry};
use crate::error::Result;
use crate::services::timer_service;
use tauri::State;

#[tauri::command]
pub fn get_active_timer(db: State<DbConnection>) -> Result<Option<ActiveTimer>> {
    timer_service::recover_stale_active_timer(db.inner())?;
    timer_service::get_active_timer(db.inner())
}

#[tauri::command]
pub fn start_timer(db: State<DbConnection>, task_id: i64) -> Result<ActiveTimer> {
    timer_service::start_timer(db.inner(), task_id)
}

#[tauri::command]
pub fn pause_timer(db: State<DbConnection>) -> Result<()> {
    timer_service::pause_timer(db.inner())
}

#[tauri::command]
pub fn resume_timer(db: State<DbConnection>) -> Result<()> {
    timer_service::resume_timer(db.inner())
}

#[tauri::command]
pub fn stop_timer(db: State<DbConnection>) -> Result<TimeEntry> {
    timer_service::stop_timer(db.inner())
}

#[tauri::command]
pub fn reset_timer(db: State<DbConnection>) -> Result<()> {
    timer_service::reset_timer(db.inner())
}

#[tauri::command]
pub fn log_break_time(db: State<DbConnection>, duration_seconds: i64) -> Result<()> {
    timer_service::log_break_time(db.inner(), duration_seconds)
}
