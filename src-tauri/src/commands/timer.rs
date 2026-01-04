use crate::db::{ActiveTimer, DbConnection, TimeEntry};
use crate::error::{AppError, Result};
use tauri::State;

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn get_active_timer(db: State<DbConnection>) -> Result<Option<ActiveTimer>> {
    let conn = db.lock();

    let result = conn.query_row(
        "SELECT t.task_id, t.started_at, t.elapsed_seconds, t.is_running, tasks.title
         FROM active_timer t
         LEFT JOIN tasks ON t.task_id = tasks.id
         WHERE t.id = 1",
        [],
        |row| {
            Ok(ActiveTimer {
                task_id: row.get(0)?,
                started_at: row.get(1)?,
                elapsed_seconds: row.get(2)?,
                is_running: row.get(3)?,
                task_title: row.get(4)?,
            })
        },
    );

    match result {
        Ok(timer) => Ok(Some(timer)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub fn start_timer(db: State<DbConnection>, task_id: i64) -> Result<ActiveTimer> {
    if let Some(existing) = get_active_timer(db.clone())? {
        return Err(AppError::TimerActive(format!(
            "Timer already running for task {}",
            existing.task_id
        )));
    }

    let conn = db.lock();

    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| Ok(true))
        .unwrap_or(false);

    if !task_exists {
        return Err(AppError::NotFound(format!("Task with id {} not found", task_id)));
    }

    let now = get_timestamp();

    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running)
         VALUES (1, ?, ?, 0, 1)",
        (task_id, now),
    )?;

    let task_title: Option<String> = conn
        .query_row("SELECT title FROM tasks WHERE id = ?", [task_id], |row| row.get(0))
        .ok();

    Ok(ActiveTimer {
        task_id,
        started_at: now,
        elapsed_seconds: 0,
        is_running: true,
        task_title,
    })
}

#[tauri::command]
pub fn pause_timer(db: State<DbConnection>) -> Result<()> {
    let timer = get_active_timer(db.clone())?
        .ok_or(AppError::NoActiveTimer)?;

    if !timer.is_running {
        return Ok(());
    }

    let conn = db.lock();

    let now = get_timestamp();
    let additional_time = now - timer.started_at;

    conn.execute(
        "UPDATE active_timer SET is_running = 0, elapsed_seconds = ? WHERE id = 1",
        [timer.elapsed_seconds + additional_time],
    )?;

    Ok(())
}

#[tauri::command]
pub fn resume_timer(db: State<DbConnection>) -> Result<()> {
    let timer = get_active_timer(db.clone())?
        .ok_or(AppError::NoActiveTimer)?;

    if timer.is_running {
        return Ok(());
    }

    let conn = db.lock();

    let now = get_timestamp();

    conn.execute(
        "UPDATE active_timer SET is_running = 1, started_at = ? WHERE id = 1",
        [now],
    )?;

    Ok(())
}

#[tauri::command]
pub fn stop_timer(db: State<DbConnection>) -> Result<TimeEntry> {
    let timer = get_active_timer(db.clone())?
        .ok_or(AppError::NoActiveTimer)?;

    let conn = db.lock();

    let now = get_timestamp();
    let total_duration = if timer.is_running {
        timer.elapsed_seconds + (now - timer.started_at)
    } else {
        timer.elapsed_seconds
    };

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (timer.task_id, total_duration, timer.started_at, now, now),
    )?;

    let entry_id = conn.last_insert_rowid();

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (total_duration, timer.task_id),
    )?;

    let project_id: i64 = conn
        .query_row("SELECT project_id FROM tasks WHERE id = ?", [timer.task_id], |row| row.get(0))?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (total_duration, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row("SELECT section_id FROM tasks WHERE id = ?", [timer.task_id], |row| row.get(0))
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (total_duration, sid),
        )?;
    }

    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;

    Ok(TimeEntry {
        id: entry_id,
        task_id: timer.task_id,
        entry_type: "timer".to_string(),
        duration_seconds: total_duration,
        started_at: Some(timer.started_at),
        ended_at: Some(now),
        note: None,
        created_at: now,
    })
}
