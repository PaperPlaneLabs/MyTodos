use crate::db::DbConnection;
use crate::error::Result;
use crate::google::GoogleCalendarState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct AuthStatus {
    pub connected: bool,
}

#[derive(Serialize)]
pub struct SyncResult {
    pub synced: i32,
    pub failed: i32,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn google_auth_start(
    google_state: State<'_, GoogleCalendarState>,
    app_handle: tauri::AppHandle,
) -> Result<String> {
    crate::google::oauth::start_auth_flow(&google_state, app_handle).await
}

#[tauri::command]
pub async fn google_auth_status(
    google_state: State<'_, GoogleCalendarState>,
) -> Result<AuthStatus> {
    Ok(AuthStatus {
        connected: google_state.is_connected().await,
    })
}

#[tauri::command]
pub async fn google_auth_disconnect(
    google_state: State<'_, GoogleCalendarState>,
    db: State<'_, DbConnection>,
) -> Result<()> {
    // Clear tokens
    crate::google::token_store::clear_tokens();
    *google_state.tokens.lock().await = None;

    // Clear all google_event_ids from tasks
    let conn = db.lock();
    conn.execute(
        "UPDATE tasks SET google_event_id = NULL WHERE google_event_id IS NOT NULL",
        [],
    )?;

    Ok(())
}

#[tauri::command]
pub async fn google_sync_all_tasks(
    google_state: State<'_, GoogleCalendarState>,
    db: State<'_, DbConnection>,
) -> Result<SyncResult> {
    let db_clone = db.inner().clone();
    let (synced, failed, errors) =
        crate::google::sync::sync_all_tasks(db_clone, &google_state).await?;

    Ok(SyncResult {
        synced,
        failed,
        errors,
    })
}
