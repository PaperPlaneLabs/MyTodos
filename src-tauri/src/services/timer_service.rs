use crate::commands::common::{apply_task_and_parent_time_delta, get_timestamp};
use crate::db::{ActiveTimer, DbConnection, TimeEntry, TimedTimerCompletion};
use crate::error::{AppError, Result};
use rusqlite::Connection;

pub const ACTIVE_TIMER_HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
pub const TIMED_TIMER_CHECK_INTERVAL_SECONDS: u64 = 1;
const ACTIVE_TIMER_STALE_AFTER_SECONDS: i64 = 120;
const MAX_TIMED_TIMER_SECONDS: i64 = 24 * 60 * 60;
use super::{
    afk_categories_service, AFK_PROJECT_COLOR, AFK_PROJECT_NAME, BREAK_PROJECT_NAME,
    BREAK_TASK_TITLE,
};

const BREAK_PROJECT_DESCRIPTION: &str = "Automatically tracked break time";
const BREAK_PROJECT_COLOR: &str = "#10b981";
const BREAK_TASK_DESCRIPTION: &str = "Auto-generated task for break time";
const AFK_PROJECT_DESCRIPTION: &str = "Automatically tracked away-from-keyboard time";

fn get_active_timer_from_conn(conn: &Connection) -> Result<Option<ActiveTimer>> {
    let result = conn.query_row(
        "SELECT t.task_id, t.started_at, t.elapsed_seconds, t.is_running,
                t.last_heartbeat_at, tasks.title, t.project_id,
                t.timer_limit_seconds, t.timer_remaining_seconds, t.timer_expires_at
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
                timer_limit_seconds: row.get(7)?,
                timer_remaining_seconds: row.get(8)?,
                timer_expires_at: row.get(9)?,
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
    let bounded_end = timer
        .timer_expires_at
        .map(|expires_at| ended_at.min(expires_at))
        .unwrap_or(ended_at);
    let safe_end = bounded_end.max(timer.started_at);
    let duration = if let Some(remaining_seconds) = timer.timer_remaining_seconds {
        (safe_end - timer.started_at).max(0).min(remaining_seconds)
    } else {
        calculate_running_duration(timer, safe_end)
    };

    if duration > 0 || timer.timer_remaining_seconds.is_none() {
        conn.execute(
            "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
             VALUES (?, 'timer', ?, ?, ?, ?)",
            (timer.task_id, duration, timer.started_at, safe_end, safe_end),
        )?;

        apply_task_and_parent_time_delta(conn, timer.task_id, duration)?;
    }

    let remaining_seconds = timer
        .timer_remaining_seconds
        .map(|remaining| (remaining - duration).max(0));

    conn.execute(
        "UPDATE active_timer
         SET is_running = 0, elapsed_seconds = 0, started_at = ?, last_heartbeat_at = ?,
             timer_remaining_seconds = ?, timer_expires_at = NULL
         WHERE id = 1",
        (safe_end, safe_end, remaining_seconds),
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

fn start_timer_with_limit(
    db: &DbConnection,
    task_id: i64,
    timer_limit_seconds: Option<i64>,
) -> Result<ActiveTimer> {
    if let Some(existing) = get_active_timer(db)? {
        return Err(AppError::TimerActive(format!(
            "Timer already running for task {}",
            existing.task_id
        )));
    }

    let conn = db.lock();
    let task_info: (Option<String>, Option<i64>, bool) = conn
        .query_row(
            "SELECT title, project_id, completed FROM tasks WHERE id = ?",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", task_id)))?;

    if timer_limit_seconds.is_some() && task_info.2 {
        return Err(AppError::InvalidInput(
            "Cannot start a task timer for a completed task".to_string(),
        ));
    }

    let now = get_timestamp();
    let timer_expires_at = timer_limit_seconds.map(|seconds| now + seconds);

    conn.execute(
        "INSERT INTO active_timer (
             id, task_id, started_at, elapsed_seconds, is_running,
             last_heartbeat_at, project_id, timer_limit_seconds,
             timer_remaining_seconds, timer_expires_at
         ) VALUES (1, ?, ?, 0, 1, ?, ?, ?, ?, ?)",
        (
            task_id,
            now,
            now,
            task_info.1,
            timer_limit_seconds,
            timer_limit_seconds,
            timer_expires_at,
        ),
    )?;

    Ok(ActiveTimer {
        task_id,
        started_at: now,
        elapsed_seconds: 0,
        is_running: true,
        last_heartbeat_at: Some(now),
        task_title: task_info.0,
        project_id: task_info.1,
        timer_limit_seconds,
        timer_remaining_seconds: timer_limit_seconds,
        timer_expires_at,
    })
}

pub fn start_timer(db: &DbConnection, task_id: i64) -> Result<ActiveTimer> {
    start_timer_with_limit(db, task_id, None)
}

pub fn start_timed_timer(
    db: &DbConnection,
    task_id: i64,
    duration_seconds: i64,
) -> Result<ActiveTimer> {
    if !(1..=MAX_TIMED_TIMER_SECONDS).contains(&duration_seconds) {
        return Err(AppError::InvalidInput(
            "Task timer duration must be between 1 second and 24 hours".to_string(),
        ));
    }

    start_timer_with_limit(db, task_id, Some(duration_seconds))
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
    let timer_expires_at = timer
        .timer_remaining_seconds
        .map(|remaining_seconds| now + remaining_seconds);

    conn.execute(
        "UPDATE active_timer
         SET is_running = 1, started_at = ?, last_heartbeat_at = ?, timer_expires_at = ?
         WHERE id = 1",
        (now, now, timer_expires_at),
    )?;

    Ok(())
}

pub fn stop_timer(db: &DbConnection) -> Result<TimeEntry> {
    let timer = get_active_timer(db)?.ok_or(AppError::NoActiveTimer)?;
    let conn = db.lock();
    let now = get_timestamp();
    let safe_end = timer
        .timer_expires_at
        .map(|expires_at| now.min(expires_at))
        .unwrap_or(now);
    let total_duration = if timer.is_running {
        if let Some(remaining_seconds) = timer.timer_remaining_seconds {
            (safe_end - timer.started_at).max(0).min(remaining_seconds)
        } else {
            calculate_running_duration(&timer, safe_end)
        }
    } else {
        timer.elapsed_seconds
    };

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (
            timer.task_id,
            total_duration,
            timer.started_at,
            safe_end,
            now,
        ),
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
        ended_at: Some(safe_end),
        note: None,
        created_at: now,
    })
}

pub fn finish_expired_timed_timer_at(
    db: &DbConnection,
    now: i64,
) -> Result<Option<TimedTimerCompletion>> {
    let mut conn = db.lock();
    let transaction = conn.transaction()?;
    let Some(timer) = get_active_timer_from_conn(&transaction)? else {
        return Ok(None);
    };
    let Some(limit_seconds) = timer.timer_limit_seconds else {
        return Ok(None);
    };
    let remaining_seconds = timer.timer_remaining_seconds.unwrap_or(limit_seconds);
    let is_due = remaining_seconds <= 0
        || (timer.is_running
            && timer
                .timer_expires_at
                .is_some_and(|expires_at| now >= expires_at));
    if !is_due {
        return Ok(None);
    }

    let finished_at = timer.timer_expires_at.unwrap_or(now).min(now);
    if timer.is_running && remaining_seconds > 0 {
        let duration = (finished_at - timer.started_at)
            .max(0)
            .min(remaining_seconds);
        if duration > 0 {
            transaction.execute(
                "INSERT INTO time_entries (
                     task_id, entry_type, duration_seconds, started_at, ended_at, created_at
                 ) VALUES (?, 'timer', ?, ?, ?, ?)",
                (
                    timer.task_id,
                    duration,
                    timer.started_at,
                    finished_at,
                    finished_at,
                ),
            )?;
            apply_task_and_parent_time_delta(&transaction, timer.task_id, duration)?;
        }
    }

    transaction.execute("DELETE FROM active_timer WHERE id = 1", [])?;
    transaction.commit()?;

    Ok(Some(TimedTimerCompletion {
        task_id: timer.task_id,
        task_title: timer.task_title.unwrap_or_else(|| "Task".to_string()),
        duration_seconds: limit_seconds,
        finished_at,
    }))
}

pub fn reset_timer(db: &DbConnection) -> Result<()> {
    let _timer = get_active_timer(db)?.ok_or(AppError::NoActiveTimer)?;
    let conn = db.lock();
    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;
    Ok(())
}

fn get_or_create_system_project(
    conn: &Connection,
    name: &str,
    description: &str,
    color: &str,
    now: i64,
) -> Result<i64> {
    match conn.query_row(
        "SELECT id
         FROM projects
         WHERE is_system = 1 AND name = ?
         ORDER BY id ASC
         LIMIT 1",
        [name],
        |row| row.get(0),
    ) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let position: i64 = conn
                .query_row(
                    "SELECT COALESCE(MAX(position), -1) + 1 FROM projects",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            conn.execute(
                "INSERT INTO projects (
                    name,
                    description,
                    color,
                    position,
                    total_time_seconds,
                    is_system,
                    created_at,
                    updated_at
                 ) VALUES (?, ?, ?, ?, 0, 1, ?, ?)",
                (name, description, color, position, now, now),
            )?;

            Ok(conn.last_insert_rowid())
        }
        Err(error) => Err(error.into()),
    }
}

fn get_or_create_system_task(
    conn: &Connection,
    project_id: i64,
    title: &str,
    description: &str,
    now: i64,
) -> Result<i64> {
    match conn.query_row(
        "SELECT id
         FROM tasks
         WHERE is_system = 1 AND project_id = ? AND title = ?
         ORDER BY id ASC
         LIMIT 1",
        (project_id, title),
        |row| row.get(0),
    ) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            conn.execute(
                "INSERT INTO tasks (
                    project_id,
                    title,
                    description,
                    total_time_seconds,
                    is_system,
                    created_at,
                    updated_at
                 ) VALUES (?, ?, ?, 0, 1, ?, ?)",
                (project_id, title, description, now, now),
            )?;

            Ok(conn.last_insert_rowid())
        }
        Err(error) => Err(error.into()),
    }
}

fn insert_manual_time_entry(
    conn: &Connection,
    task_id: i64,
    duration_seconds: i64,
    now: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'manual', ?, ?, ?, ?)",
        (task_id, duration_seconds, now - duration_seconds, now, now),
    )?;

    apply_task_and_parent_time_delta(conn, task_id, duration_seconds)?;
    Ok(())
}

pub fn log_break_time(db: &DbConnection, duration_seconds: i64) -> Result<()> {
    if duration_seconds <= 0 {
        return Ok(());
    }

    let conn = db.lock();
    let now = get_timestamp();
    let project_id = get_or_create_system_project(
        &conn,
        BREAK_PROJECT_NAME,
        BREAK_PROJECT_DESCRIPTION,
        BREAK_PROJECT_COLOR,
        now,
    )?;
    let task_id = get_or_create_system_task(
        &conn,
        project_id,
        BREAK_TASK_TITLE,
        BREAK_TASK_DESCRIPTION,
        now,
    )?;

    insert_manual_time_entry(&conn, task_id, duration_seconds, now)?;
    Ok(())
}

pub fn log_afk_time(db: &DbConnection, category_name: &str, duration_seconds: i64) -> Result<()> {
    if duration_seconds <= 0 {
        return Ok(());
    }

    let conn = db.lock();
    let saved_name = afk_categories_service::ensure_category_in_conn(&conn, category_name)?;
    let now = get_timestamp();
    let project_id = get_or_create_system_project(
        &conn,
        AFK_PROJECT_NAME,
        AFK_PROJECT_DESCRIPTION,
        AFK_PROJECT_COLOR,
        now,
    )?;
    let task_description = format!("Auto-generated task for {} away time", saved_name);
    let task_id =
        get_or_create_system_task(&conn, project_id, &saved_name, &task_description, now)?;

    insert_manual_time_entry(&conn, task_id, duration_seconds, now)?;
    Ok(())
}
