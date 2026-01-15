#[tauri::command]
pub fn enable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    tauri_plugin_autostart::enable(app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    tauri_plugin_autostart::disable(app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    tauri_plugin_autostart::is_enabled(app).map_err(|e| e.to_string())
}
