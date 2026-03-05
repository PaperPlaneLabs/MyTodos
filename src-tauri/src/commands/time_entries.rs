use super::common::{apply_task_and_parent_time_delta, get_timestamp};
use crate::db::{DbConnection, TimeEntry, TimeEntryWithTask};
use crate::error::{AppError, Result};
use tauri::State;

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
        return Err(AppError::NotFound(format!(
            "Task with id {} not found",
            task_id
        )));
    }

    if duration_seconds <= 0 {
        return Err(AppError::InvalidInput(
            "Duration must be positive".to_string(),
        ));
    }

    let now = get_timestamp();

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, note, created_at)
         VALUES (?, 'manual', ?, ?, ?)",
        (task_id, duration_seconds, &note, now),
    )?;

    let id = conn.last_insert_rowid();

    apply_task_and_parent_time_delta(&conn, task_id, duration_seconds)?;

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
         FROM time_entries WHERE task_id = ? ORDER BY created_at DESC",
    )?;

    let entries = stmt
        .query_map([task_id], |row| {
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
        return Err(AppError::InvalidInput(
            "Duration must be positive".to_string(),
        ));
    }

    let old_duration: i64 = conn
        .query_row(
            "SELECT duration_seconds FROM time_entries WHERE id = ?",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::NotFound(format!("Time entry with id {} not found", id)))?;

    let task_id: i64 = conn.query_row(
        "SELECT task_id FROM time_entries WHERE id = ?",
        [id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE time_entries SET duration_seconds = ?, note = ? WHERE id = ?",
        (duration_seconds, note, id),
    )?;

    let diff = duration_seconds - old_duration;

    apply_task_and_parent_time_delta(&conn, task_id, diff)?;

    Ok(())
}

#[tauri::command]
pub fn delete_time_entry(db: State<DbConnection>, id: i64) -> Result<()> {
    let conn = db.lock();

    let (task_id, duration): (i64, i64) = conn
        .query_row(
            "SELECT task_id, duration_seconds FROM time_entries WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Time entry with id {} not found", id)))?;

    conn.execute("DELETE FROM time_entries WHERE id = ?", [id])?;

    apply_task_and_parent_time_delta(&conn, task_id, -duration)?;

    Ok(())
}
#[tauri::command]
pub fn get_daily_total_time(db: State<DbConnection>, start_timestamp: i64) -> Result<i64> {
    let conn = db.lock();
    // end_timestamp = start_timestamp + 86400 (one full day)
    let end_timestamp = start_timestamp + 86400;

    // For timer entries (with started_at + ended_at): clip the session to [start, end] and
    // sum only the overlapping seconds, so a session that spans midnight doesn't inflate today.
    // For manual entries (no started_at): include only if created today, trust duration_seconds.
    let total: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
            CASE
                WHEN started_at IS NOT NULL AND ended_at IS NOT NULL
                THEN MAX(0, MIN(ended_at, ?) - MAX(started_at, ?))
                ELSE CASE WHEN created_at >= ? AND created_at < ? THEN duration_seconds ELSE 0 END
            END
        ), 0)
        FROM time_entries
        WHERE
            (started_at IS NOT NULL AND ended_at IS NOT NULL AND started_at < ? AND ended_at > ?)
            OR (started_at IS NULL AND created_at >= ? AND created_at < ?)",
        [
            end_timestamp,
            start_timestamp,
            start_timestamp,
            end_timestamp,
            end_timestamp,
            start_timestamp,
            start_timestamp,
            end_timestamp,
        ],
        |row| row.get(0),
    )?;

    Ok(total)
}

#[tauri::command]
pub fn get_time_entries_with_tasks(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<TimeEntryWithTask>> {
    let conn = db.lock();

    let mut stmt = conn.prepare(
        "SELECT
            te.id,
            te.task_id,
            t.title as task_title,
            t.project_id,
            p.name as project_name,
            p.color as project_color,
            te.duration_seconds,
            COALESCE(te.started_at, te.created_at) as started_at,
            COALESCE(te.ended_at, te.created_at + te.duration_seconds) as ended_at,
            te.note
         FROM time_entries te
         JOIN tasks t ON te.task_id = t.id
         LEFT JOIN projects p ON t.project_id = p.id
         WHERE date(te.created_at, 'unixepoch') BETWEEN ? AND ?
         ORDER BY started_at ASC",
    )?;

    let entries = stmt
        .query_map([start_date.clone(), end_date.clone()], |row| {
            Ok(TimeEntryWithTask {
                id: row.get(0)?,
                task_id: row.get(1)?,
                task_title: row.get(2)?,
                project_id: row.get(3)?,
                project_name: row.get(4)?,
                project_color: row.get(5)?,
                duration_seconds: row.get(6)?,
                started_at: row.get(7)?,
                ended_at: row.get(8)?,
                note: row.get(9)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(entries)
}
