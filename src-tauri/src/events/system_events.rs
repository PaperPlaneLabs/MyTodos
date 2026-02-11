use crate::db::{ActiveTimer, DbConnection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Reasons for auto-pausing the timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoPauseReason {
    SystemSleep,
    ScreenLock,
    Shutdown,
}

/// Event emitted to frontend when timer is auto-paused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPauseEvent {
    pub reason: AutoPauseReason,
    pub timestamp: i64,
}

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

fn get_active_timer_internal(db: &DbConnection) -> crate::error::Result<Option<ActiveTimer>> {
    let conn = db.lock();

    let result = conn.query_row(
        "SELECT t.task_id, t.started_at, t.elapsed_seconds, t.is_running, tasks.title, t.project_id
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
                project_id: row.get(5)?,
            })
        },
    );

    match result {
        Ok(timer) => Ok(Some(timer)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn pause_timer_internal(db: &DbConnection, timer: &ActiveTimer) -> crate::error::Result<()> {
    let conn = db.lock();

    let now = get_timestamp();
    let duration = timer.elapsed_seconds + (now - timer.started_at);

    // Create a time entry for the paused duration
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (timer.task_id, duration, timer.started_at, now, now),
    )?;

    // Update task total
    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration, timer.task_id),
    )?;

    // Update project total
    let project_id: i64 = conn.query_row(
        "SELECT project_id FROM tasks WHERE id = ?",
        [timer.task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration, project_id),
    )?;

    // Update section total if applicable
    let section_id: Option<i64> = conn
        .query_row(
            "SELECT section_id FROM tasks WHERE id = ?",
            [timer.task_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (duration, sid),
        )?;
    }

    // Reset elapsed_seconds to 0 and update active_timer
    conn.execute(
        "UPDATE active_timer SET is_running = 0, elapsed_seconds = 0, started_at = ? WHERE id = 1",
        [now],
    )?;

    Ok(())
}

/// Auto-pause the timer if it's currently running
///
/// This function checks if there's an active timer and pauses it if running.
/// It then emits an event to the frontend to update the UI.
///
/// # Arguments
/// * `app_handle` - Tauri app handle for emitting events
/// * `db` - Database connection
/// * `reason` - The reason for auto-pausing
pub fn auto_pause_if_running(
    app_handle: &AppHandle,
    db: &DbConnection,
    reason: AutoPauseReason,
) {
    // Check if timer exists and is running
    match get_active_timer_internal(db) {
        Ok(Some(timer)) if timer.is_running => {
            // Pause the timer
            match pause_timer_internal(db, &timer) {
                Ok(_) => {
                    // Emit event to frontend
                    let event = AutoPauseEvent {
                        reason: reason.clone(),
                        timestamp: chrono::Utc::now().timestamp(),
                    };

                    if let Err(e) = app_handle.emit("timer:auto-paused", event) {
                        eprintln!("Failed to emit auto-pause event: {}", e);
                    } else {
                        println!("Timer auto-paused due to {:?}", reason);
                    }
                }
                Err(e) => eprintln!("Failed to auto-pause timer: {}", e),
            }
        }
        Ok(Some(_)) => {
            // Timer exists but not running - no action needed
        }
        Ok(None) => {
            // No active timer - no action needed
        }
        Err(e) => eprintln!("Failed to check active timer: {}", e),
    }
}
