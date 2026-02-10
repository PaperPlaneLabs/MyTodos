use crate::db::DbConnection;
use crate::error::{AppError, Result};
use super::{calendar_api, oauth, GoogleCalendarState};

/// Sync a single task to Google Calendar.
/// - If task has deadline and no google_event_id -> create event
/// - If task has deadline and google_event_id -> update event
/// - If task has no deadline and google_event_id -> delete event and clear ID
pub async fn sync_task_to_calendar(
    db: DbConnection,
    google_state: &GoogleCalendarState,
    task_id: i64,
) -> Result<()> {
    if !google_state.is_connected().await {
        return Ok(());
    }

    // Read task data from DB
    let (title, description, deadline, google_event_id) = {
        let conn = db.lock();
        conn.query_row(
            "SELECT title, description, deadline, google_event_id FROM tasks WHERE id = ?",
            [task_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            },
        )
        .map_err(|e| AppError::Other(format!("Failed to read task {}: {}", task_id, e)))?
    };

    let access_token = oauth::refresh_token_if_needed(google_state).await?;

    match (deadline.as_deref(), google_event_id.as_deref()) {
        // Has deadline, no event yet -> create
        (Some(date), None) => {
            let event_id = calendar_api::create_event(
                &access_token,
                &title,
                description.as_deref(),
                date,
            )
            .await?;

            // Store event ID in DB
            let conn = db.lock();
            conn.execute(
                "UPDATE tasks SET google_event_id = ? WHERE id = ?",
                rusqlite::params![event_id, task_id],
            )
            .map_err(|e| AppError::Other(format!("Failed to store event ID: {}", e)))?;
        }
        // Has deadline and event -> update
        (Some(date), Some(event_id)) => {
            calendar_api::update_event(
                &access_token,
                event_id,
                &title,
                description.as_deref(),
                date,
            )
            .await?;
        }
        // No deadline but has event -> delete
        (None, Some(event_id)) => {
            calendar_api::delete_event(&access_token, event_id).await?;

            let conn = db.lock();
            conn.execute(
                "UPDATE tasks SET google_event_id = NULL WHERE id = ?",
                [task_id],
            )
            .map_err(|e| AppError::Other(format!("Failed to clear event ID: {}", e)))?;
        }
        // No deadline, no event -> nothing to do
        (None, None) => {}
    }

    Ok(())
}

/// Delete an event from Google Calendar by event ID.
/// Used when a task is deleted or completed.
pub async fn delete_from_calendar(
    google_state: &GoogleCalendarState,
    google_event_id: &str,
) -> Result<()> {
    if !google_state.is_connected().await {
        return Ok(());
    }

    let access_token = oauth::refresh_token_if_needed(google_state).await?;
    calendar_api::delete_event(&access_token, google_event_id).await?;

    Ok(())
}

/// Sync all tasks with deadlines to Google Calendar.
/// Returns (synced_count, failed_count, errors).
pub async fn sync_all_tasks(
    db: DbConnection,
    google_state: &GoogleCalendarState,
) -> Result<(i32, i32, Vec<String>)> {
    if !google_state.is_connected().await {
        return Err(AppError::GoogleAuth("Not connected to Google".to_string()));
    }

    // Get all tasks with deadlines that are not completed
    let task_ids: Vec<i64> = {
        let conn = db.lock();
        let mut stmt = conn
            .prepare("SELECT id FROM tasks WHERE deadline IS NOT NULL AND completed = 0")
            .map_err(|e| AppError::Other(format!("Failed to query tasks: {}", e)))?;
        let ids: Vec<i64> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| AppError::Other(format!("Failed to map tasks: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();
        ids
    };

    let mut synced = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for task_id in task_ids {
        match sync_task_to_calendar(db.clone(), google_state, task_id).await {
            Ok(()) => synced += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Task {}: {}", task_id, e));
            }
        }
    }

    Ok((synced, failed, errors))
}
