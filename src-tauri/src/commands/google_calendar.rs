use crate::db::{
    models::{GoogleCalendarEvent, GoogleCalendarRange},
    DbConnection,
};
use crate::error::Result;
use crate::google::GoogleCalendarState;
use serde::Serialize;
use std::collections::HashSet;
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
    conn.execute("DELETE FROM google_calendar_event_cache", [])?;

    Ok(())
}

fn cached_events(
    db: &DbConnection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<GoogleCalendarEvent>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT external_id, title, description, is_all_day, start_date, end_date,
                start_at, end_at, timezone, html_link, color
         FROM google_calendar_event_cache
         WHERE start_date <= ?2 AND end_date > ?1
         ORDER BY start_date, start_at, title",
    )?;
    let events = stmt
        .query_map(rusqlite::params![start_date, end_date], |row| {
            Ok(GoogleCalendarEvent {
                external_id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                is_all_day: row.get(3)?,
                start_date: row.get(4)?,
                end_date: row.get(5)?,
                start_at: row.get(6)?,
                end_at: row.get(7)?,
                timezone: row.get(8)?,
                html_link: row.get(9)?,
                color: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(events)
}

fn cache_events(
    db: &DbConnection,
    start_date: &str,
    end_date: &str,
    events: &[GoogleCalendarEvent],
) -> Result<()> {
    let mut conn = db.lock();
    let transaction = conn.transaction()?;
    transaction.execute(
        "DELETE FROM google_calendar_event_cache
         WHERE start_date <= ?2 AND end_date > ?1",
        rusqlite::params![start_date, end_date],
    )?;
    let now = chrono::Utc::now().timestamp();
    for event in events {
        transaction.execute(
            "INSERT INTO google_calendar_event_cache
             (external_id, title, description, is_all_day, start_date, end_date,
              start_at, end_at, timezone, html_link, color, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(external_id) DO UPDATE SET
               title = excluded.title, description = excluded.description,
               is_all_day = excluded.is_all_day, start_date = excluded.start_date,
               end_date = excluded.end_date, start_at = excluded.start_at,
               end_at = excluded.end_at, timezone = excluded.timezone,
               html_link = excluded.html_link, color = excluded.color,
               updated_at = excluded.updated_at",
            rusqlite::params![
                event.external_id,
                event.title,
                event.description,
                event.is_all_day,
                event.start_date,
                event.end_date,
                event.start_at,
                event.end_at,
                event.timezone,
                event.html_link,
                event.color,
                now,
            ],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

#[tauri::command]
pub async fn google_get_events_in_range(
    google_state: State<'_, GoogleCalendarState>,
    db: State<'_, DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<GoogleCalendarRange> {
    if !google_state.is_connected().await {
        return Ok(GoogleCalendarRange {
            events: Vec::new(),
            stale: false,
            error: None,
        });
    }

    let refresh = async {
        let access_token = crate::google::oauth::refresh_token_if_needed(&google_state).await?;
        let mut events =
            crate::google::calendar_api::list_events(&access_token, &start_date, &end_date).await?;
        let exported_ids: HashSet<String> = {
            let conn = db.lock();
            let mut stmt = conn
                .prepare("SELECT google_event_id FROM tasks WHERE google_event_id IS NOT NULL")?;
            let ids = stmt
                .query_map([], |row| row.get(0))?
                .collect::<std::result::Result<HashSet<_>, _>>()?;
            ids
        };
        events.retain(|event| !exported_ids.contains(&event.external_id));
        cache_events(db.inner(), &start_date, &end_date, &events)?;
        Result::<Vec<GoogleCalendarEvent>>::Ok(events)
    }
    .await;

    match refresh {
        Ok(events) => Ok(GoogleCalendarRange {
            events,
            stale: false,
            error: None,
        }),
        Err(error) => Ok(GoogleCalendarRange {
            events: cached_events(db.inner(), &start_date, &end_date)?,
            stale: true,
            error: Some(error.to_string()),
        }),
    }
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
