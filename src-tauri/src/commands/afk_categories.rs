use crate::db::DbConnection;
use crate::error::Result;
use crate::services::afk_categories_service;
use tauri::State;

#[tauri::command]
pub fn get_afk_categories(db: State<DbConnection>) -> Result<Vec<String>> {
    afk_categories_service::list_categories(db.inner())
}

#[tauri::command]
pub fn add_afk_category(db: State<DbConnection>, name: String) -> Result<String> {
    afk_categories_service::add_category(db.inner(), &name)
}

#[tauri::command]
pub fn merge_afk_categories(db: State<DbConnection>, names: Vec<String>) -> Result<Vec<String>> {
    afk_categories_service::merge_categories(db.inner(), names)
}

#[tauri::command]
pub fn remove_afk_category(db: State<DbConnection>, name: String) -> Result<()> {
    afk_categories_service::remove_category(db.inner(), &name)
}
