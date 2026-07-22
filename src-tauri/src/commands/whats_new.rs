use crate::db::DbConnection;
use crate::error::Result;
use crate::services::whats_new_service;
use tauri::State;

#[tauri::command]
pub fn get_whats_new_last_seen_version(db: State<DbConnection>) -> Result<Option<String>> {
    whats_new_service::get_last_seen_version(db.inner())
}

#[tauri::command]
pub fn set_whats_new_last_seen_version(db: State<DbConnection>, version: String) -> Result<()> {
    whats_new_service::set_last_seen_version(db.inner(), &version)
}
