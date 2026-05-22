use crate::db::DbConnection;
use crate::error::Result;
use crate::services::backup_service::{self, BackupSettings};
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn get_backup_settings(db: State<DbConnection>) -> Result<BackupSettings> {
    backup_service::get_settings(db.inner())
}

#[tauri::command]
pub fn set_backup_settings(
    db: State<DbConnection>,
    enabled: bool,
    folder: String,
    interval_minutes: i64,
) -> Result<BackupSettings> {
    backup_service::set_settings(db.inner(), enabled, folder, interval_minutes)
}

#[tauri::command]
pub fn backup_now(db: State<DbConnection>) -> Result<BackupSettings> {
    backup_service::backup_now(db.inner())
}

#[tauri::command]
pub fn restore_backup(db: State<DbConnection>, backup_path: String) -> Result<()> {
    backup_service::restore_backup(db.inner(), Path::new(&backup_path))
}

#[tauri::command]
pub fn check_cloud_folders() -> Vec<String> {
    backup_service::check_cloud_folders()
}
