use super::common::{apply_parent_time_delta, get_timestamp};
use crate::db::{DbConnection, Task};
use crate::error::{AppError, Result};
use crate::google::GoogleCalendarState;
use tauri::State;

#[tauri::command]
pub fn get_tasks_by_project(db: State<DbConnection>, project_id: i64) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, google_event_id, created_at, updated_at
         FROM tasks WHERE project_id = ? ORDER BY position ASC"
    )?;

    let tasks = stmt
        .query_map([project_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                section_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                completed: row.get(5)?,
                position: row.get(6)?,
                total_time_seconds: row.get(7)?,
                deadline: row.get(8)?,
                google_event_id: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

#[tauri::command]
pub fn get_unassigned_tasks(db: State<DbConnection>) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, google_event_id, created_at, updated_at
         FROM tasks WHERE project_id IS NULL ORDER BY position ASC"
    )?;

    let tasks = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                section_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                completed: row.get(5)?,
                position: row.get(6)?,
                total_time_seconds: row.get(7)?,
                deadline: row.get(8)?,
                google_event_id: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

#[tauri::command]
pub fn get_tasks_by_section(db: State<DbConnection>, section_id: i64) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, section_id, title, description, completed, position, total_time_seconds, deadline, google_event_id, created_at, updated_at
         FROM tasks WHERE section_id = ? ORDER BY position ASC"
    )?;

    let tasks = stmt
        .query_map([section_id], |row| {
            Ok(Task {
                id: row.get(0)?,
                project_id: row.get(1)?,
                section_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                completed: row.get(5)?,
                position: row.get(6)?,
                total_time_seconds: row.get(7)?,
                deadline: row.get(8)?,
                google_event_id: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(tasks)
}

#[tauri::command]
pub fn create_task(
    db: State<DbConnection>,
    project_id: Option<i64>,
    section_id: Option<i64>,
    title: String,
    description: Option<String>,
) -> Result<Task> {
    let conn = db.lock();
    let now = get_timestamp();

    let max_position: i32 = if let Some(sid) = section_id {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE section_id = ?",
            [sid],
            |row| row.get(0)
        )
    } else if let Some(pid) = project_id {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE project_id = ? AND section_id IS NULL",
            [pid],
            |row| row.get(0)
        )
    } else {
        conn.query_row(
            "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE project_id IS NULL AND section_id IS NULL",
            [],
            |row| row.get(0)
        )
    }
    .unwrap_or(0);

    conn.execute(
        "INSERT INTO tasks (project_id, section_id, title, description, position, deadline, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        (
            project_id,
            section_id,
            &title,
            &description,
            max_position + 1,
            Option::<String>::None,
            now,
            now,
        ),
    )?;

    let id = conn.last_insert_rowid();

    Ok(Task {
        id,
        project_id,
        section_id,
        title,
        description,
        completed: false,
        position: max_position + 1,
        total_time_seconds: 0,
        deadline: None,
        google_event_id: None,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_task(
    db: State<DbConnection>,
    google_state: State<GoogleCalendarState>,
    id: i64,
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
) -> Result<()> {
    let conn = db.lock();
    let now = get_timestamp();

    let mut stmt = conn
        .prepare("SELECT title, description, completed, google_event_id FROM tasks WHERE id = ?")?;

    let (current_title, current_description, current_completed, google_event_id) = stmt
        .query_row([id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    let new_title = title.unwrap_or(current_title.clone());
    let title_changed = new_title != current_title;

    conn.execute(
        "UPDATE tasks SET title = ?, description = ?, completed = ?, updated_at = ? WHERE id = ?",
        (
            &new_title,
            description.or(current_description),
            completed.unwrap_or(current_completed),
            now,
            id,
        ),
    )?;

    drop(stmt);
    drop(conn);

    // If title changed and task has a google event, sync to update the title
    if title_changed && google_event_id.is_some() {
        let db = db.inner().clone();
        let google_state = google_state.inner().clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = crate::google::sync::sync_task_to_calendar(db, &google_state, id).await
            {
                eprintln!("Failed to sync task title update to Google Calendar: {}", e);
            }
        });
    }

    Ok(())
}

#[tauri::command]
pub fn delete_task(
    db: State<DbConnection>,
    google_state: State<GoogleCalendarState>,
    id: i64,
) -> Result<()> {
    let conn = db.lock();

    // Capture google_event_id before deleting
    let google_event_id: Option<String> = conn
        .query_row(
            "SELECT google_event_id FROM tasks WHERE id = ?",
            [id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let rows = conn.execute("DELETE FROM tasks WHERE id = ?", [id])?;

    if rows == 0 {
        return Err(AppError::NotFound(format!("Task with id {} not found", id)));
    }

    drop(conn);

    // Fire-and-forget: delete from Google Calendar
    if let Some(event_id) = google_event_id {
        let google_state = google_state.inner().clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) =
                crate::google::sync::delete_from_calendar(&google_state, &event_id).await
            {
                eprintln!("Failed to delete Google Calendar event: {}", e);
            }
        });
    }

    Ok(())
}

#[tauri::command]
pub fn toggle_task_completion(
    db: State<DbConnection>,
    google_state: State<GoogleCalendarState>,
    id: i64,
) -> Result<bool> {
    let conn = db.lock();
    let now = get_timestamp();

    let (completed, google_event_id): (bool, Option<String>) = conn
        .query_row(
            "SELECT completed, google_event_id FROM tasks WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    let new_completed = !completed;

    conn.execute(
        "UPDATE tasks SET completed = ?, updated_at = ? WHERE id = ?",
        (new_completed, now, id),
    )?;

    // If marking complete and has a Google Calendar event, delete it
    if new_completed {
        if let Some(event_id) = google_event_id {
            conn.execute("UPDATE tasks SET google_event_id = NULL WHERE id = ?", [id])?;
            drop(conn);

            let google_state = google_state.inner().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) =
                    crate::google::sync::delete_from_calendar(&google_state, &event_id).await
                {
                    eprintln!("Failed to delete Google Calendar event on complete: {}", e);
                }
            });

            return Ok(new_completed);
        }
    }

    Ok(new_completed)
}

#[tauri::command]
pub fn reorder_tasks(db: State<DbConnection>, task_ids: Vec<i64>) -> Result<()> {
    let conn = db.lock();

    for (position, id) in task_ids.iter().enumerate() {
        conn.execute(
            "UPDATE tasks SET position = ? WHERE id = ?",
            (position as i32, *id),
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn reset_task_time(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();

    // Get current task total time
    let current_time: i64 = conn
        .query_row(
            "SELECT total_time_seconds FROM tasks WHERE id = ?",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    // Delete all time entries for this task
    conn.execute("DELETE FROM time_entries WHERE task_id = ?", [id])?;

    // Reset task's total time
    conn.execute("UPDATE tasks SET total_time_seconds = 0 WHERE id = ?", [id])?;

    // Update parent totals (project/section) to keep aggregates consistent.
    apply_parent_time_delta(&conn, id, -current_time)?;

    Ok(())
}
