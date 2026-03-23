use crate::commands::common::{apply_task_and_parent_time_delta, get_timestamp};
use crate::db::{ActiveTimer, DbConnection, TimeEntry};
use crate::error::{AppError, Result};
use rusqlite::Connection;

pub const ACTIVE_TIMER_HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
const ACTIVE_TIMER_STALE_AFTER_SECONDS: i64 = 120;

fn get_active_timer_from_conn(conn: &Connection) -> Result<Option<ActiveTimer>> {
    let result = conn.query_row(
        "SELECT t.task_id, t.started_at, t.elapsed_seconds, t.is_running, t.last_heartbeat_at, tasks.title, t.project_id
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
                last_heartbeat_at: row.get(4)?,
                task_title: row.get(5)?,
                project_id: row.get(6)?,
            })
        },
    );

    match result {
        Ok(timer) => Ok(Some(timer)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn get_effective_last_heartbeat(timer: &ActiveTimer) -> i64 {
    timer.last_heartbeat_at.unwrap_or(timer.started_at)
}

fn calculate_running_duration(timer: &ActiveTimer, ended_at: i64) -> i64 {
    timer.elapsed_seconds + (ended_at - timer.started_at).max(0)
}

pub fn pause_running_timer_at(conn: &Connection, timer: &ActiveTimer, ended_at: i64) -> Result<()> {
    let safe_end = ended_at.max(timer.started_at);
    let duration = calculate_running_duration(timer, safe_end);

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (timer.task_id, duration, timer.started_at, safe_end, safe_end),
    )?;

    apply_task_and_parent_time_delta(conn, timer.task_id, duration)?;

    conn.execute(
        "UPDATE active_timer
         SET is_running = 0, elapsed_seconds = 0, started_at = ?, last_heartbeat_at = ?
         WHERE id = 1",
        [safe_end, safe_end],
    )?;

    Ok(())
}

pub fn get_active_timer(db: &DbConnection) -> Result<Option<ActiveTimer>> {
    let conn = db.lock();
    get_active_timer_from_conn(&conn)
}

pub fn recover_stale_active_timer(db: &DbConnection) -> Result<bool> {
    let conn = db.lock();
    let Some(timer) = get_active_timer_from_conn(&conn)? else {
        return Ok(false);
    };

    if !timer.is_running {
        return Ok(false);
    }

    let last_heartbeat = get_effective_last_heartbeat(&timer);
    let now = get_timestamp();
    if now - last_heartbeat <= ACTIVE_TIMER_STALE_AFTER_SECONDS {
        return Ok(false);
    }

    pause_running_timer_at(&conn, &timer, last_heartbeat)?;
    Ok(true)
}

pub fn heartbeat_active_timer(db: &DbConnection) -> Result<bool> {
    let conn = db.lock();
    let now = get_timestamp();
    let updated = conn.execute(
        "UPDATE active_timer
         SET last_heartbeat_at = ?
         WHERE id = 1 AND is_running = 1",
        [now],
    )?;

    Ok(updated > 0)
}

pub fn start_timer(db: &DbConnection, task_id: i64) -> Result<ActiveTimer> {
    if let Some(existing) = get_active_timer(db)? {
        return Err(AppError::TimerActive(format!(
            "Timer already running for task {}",
            existing.task_id
        )));
    }

    let conn = db.lock();
    let task_info: (Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT title, project_id FROM tasks WHERE id = ?",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", task_id)))?;

    let now = get_timestamp();

    conn.execute(
        "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, last_heartbeat_at, project_id)
         VALUES (1, ?, ?, 0, 1, ?, ?)",
        (task_id, now, now, task_info.1),
    )?;

    Ok(ActiveTimer {
        task_id,
        started_at: now,
        elapsed_seconds: 0,
        is_running: true,
        last_heartbeat_at: Some(now),
        task_title: task_info.0,
        project_id: task_info.1,
    })
}

pub fn pause_timer(db: &DbConnection) -> Result<()> {
    let timer = get_active_timer(db)?.ok_or(AppError::NoActiveTimer)?;

    if !timer.is_running {
        return Ok(());
    }

    let conn = db.lock();
    pause_running_timer_at(&conn, &timer, get_timestamp())
}

pub fn resume_timer(db: &DbConnection) -> Result<()> {
    let timer = get_active_timer(db)?.ok_or(AppError::NoActiveTimer)?;

    if timer.is_running {
        return Ok(());
    }

    let conn = db.lock();
    let now = get_timestamp();

    conn.execute(
        "UPDATE active_timer
         SET is_running = 1, started_at = ?, last_heartbeat_at = ?
         WHERE id = 1",
        [now, now],
    )?;

    Ok(())
}

pub fn stop_timer(db: &DbConnection) -> Result<TimeEntry> {
    let timer = get_active_timer(db)?.ok_or(AppError::NoActiveTimer)?;
    let conn = db.lock();
    let now = get_timestamp();
    let total_duration = if timer.is_running {
        calculate_running_duration(&timer, now)
    } else {
        timer.elapsed_seconds
    };

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (timer.task_id, total_duration, timer.started_at, now, now),
    )?;

    let entry_id = conn.last_insert_rowid();

    apply_task_and_parent_time_delta(&conn, timer.task_id, total_duration)?;
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

pub fn reset_timer(db: &DbConnection) -> Result<()> {
    let _timer = get_active_timer(db)?.ok_or(AppError::NoActiveTimer)?;
    let conn = db.lock();
    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;
    Ok(())
}

pub fn log_break_time(db: &DbConnection, duration_seconds: i64) -> Result<()> {
    if duration_seconds <= 0 {
        return Ok(());
    }

    let conn = db.lock();
    let now = get_timestamp();

    let project_id: i64 =
        match conn.query_row("SELECT id FROM projects WHERE name = 'Breaks'", [], |row| {
            row.get(0)
        }) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                let position: i64 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(position), -1) + 1 FROM projects",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                conn.execute(
                "INSERT INTO projects (name, description, color, position, created_at, updated_at) 
                     VALUES ('Breaks', 'Automatically tracked break time', '#10b981', ?, ?, ?)",
                (position, now, now),
            )?;
                conn.last_insert_rowid()
            }
            Err(error) => return Err(error.into()),
        };

    let task_id: i64 = match conn.query_row(
        "SELECT id FROM tasks WHERE project_id = ? AND title = 'Break'",
        [project_id],
        |row| row.get(0),
    ) {
        Ok(id) => id,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            conn.execute(
                "INSERT INTO tasks (project_id, title, description, created_at, updated_at) 
                 VALUES (?, 'Break', 'Auto-generated task for break time', ?, ?)",
                (project_id, now, now),
            )?;
            conn.last_insert_rowid()
        }
        Err(error) => return Err(error.into()),
    };

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'manual', ?, ?, ?, ?)",
        (task_id, duration_seconds, now - duration_seconds, now, now),
    )?;

    apply_task_and_parent_time_delta(&conn, task_id, duration_seconds)?;
    Ok(())
}
