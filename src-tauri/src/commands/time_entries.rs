use crate::db::{DbConnection, TimeEntry};
use crate::error::{AppError, Result};
use tauri::State;

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn create_manual_entry(
    db: State<DbConnection>,
    task_id: i64,
    duration_seconds: i64,
    note: Option<String>,
) -> Result<TimeEntry> {
    let conn = db.lock();

    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| Ok(true))
        .unwrap_or(false);

    if !task_exists {
        return Err(AppError::NotFound(format!("Task with id {} not found", task_id)));
    }

    if duration_seconds <= 0 {
        return Err(AppError::InvalidInput("Duration must be positive".to_string()));
    }

    let now = get_timestamp();

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, note, created_at)
         VALUES (?, 'manual', ?, ?, ?)",
        (task_id, duration_seconds, &note, now),
    )?;

    let id = conn.last_insert_rowid();

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration_seconds, task_id),
    )?;

    let project_id: i64 = conn
        .query_row("SELECT project_id FROM tasks WHERE id = ?", [task_id], |row| row.get(0))?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration_seconds, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row("SELECT section_id FROM tasks WHERE id = ?", [task_id], |row| row.get(0))
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (duration_seconds, sid),
        )?;
    }

    Ok(TimeEntry {
        id,
        task_id,
        entry_type: "manual".to_string(),
        duration_seconds,
        started_at: None,
        ended_at: None,
        note,
        created_at: now,
    })
}

#[tauri::command]
pub fn get_time_entries_by_task(db: State<DbConnection>, task_id: i64) -> Result<Vec<TimeEntry>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, task_id, entry_type, duration_seconds, started_at, ended_at, note, created_at
         FROM time_entries WHERE task_id = ? ORDER BY created_at DESC"
    )?;

    let entries = stmt.query_map([task_id], |row| {
        Ok(TimeEntry {
            id: row.get(0)?,
            task_id: row.get(1)?,
            entry_type: row.get(2)?,
            duration_seconds: row.get(3)?,
            started_at: row.get(4)?,
            ended_at: row.get(5)?,
            note: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(entries)
}

#[tauri::command]
pub fn update_time_entry(
    db: State<DbConnection>,
    id: i64,
    duration_seconds: i64,
    note: Option<String>,
) -> Result<()> {
    let conn = db.lock();

    if duration_seconds <= 0 {
        return Err(AppError::InvalidInput("Duration must be positive".to_string()));
    }

    let old_duration: i64 = conn
        .query_row("SELECT duration_seconds FROM time_entries WHERE id = ?", [id], |row| row.get(0))
        .map_err(|_| AppError::NotFound(format!("Time entry with id {} not found", id)))?;

    let task_id: i64 = conn
        .query_row("SELECT task_id FROM time_entries WHERE id = ?", [id], |row| row.get(0))?;

    conn.execute(
        "UPDATE time_entries SET duration_seconds = ?, note = ? WHERE id = ?",
        (duration_seconds, note, id),
    )?;

    let diff = duration_seconds - old_duration;

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (diff, task_id),
    )?;

    let project_id: i64 = conn
        .query_row("SELECT project_id FROM tasks WHERE id = ?", [task_id], |row| row.get(0))?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (diff, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row("SELECT section_id FROM tasks WHERE id = ?", [task_id], |row| row.get(0))
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (diff, sid),
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_time_entry(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();

    let (task_id, duration): (i64, i64) = conn
        .query_row(
            "SELECT task_id, duration_seconds FROM time_entries WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?))
        )
        .map_err(|_| AppError::NotFound(format!("Time entry with id {} not found", id)))?;

    conn.execute("DELETE FROM time_entries WHERE id = ?", [id])?;

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
        (duration, task_id),
    )?;

    let project_id: i64 = conn
        .query_row("SELECT project_id FROM tasks WHERE id = ?", [task_id], |row| row.get(0))?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
        (duration, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row("SELECT section_id FROM tasks WHERE id = ?", [task_id], |row| row.get(0))
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
            (duration, sid),
        )?;
    }

    Ok(())
}
