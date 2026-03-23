use crate::db::{ActiveTimer, DbConnection};
use crate::services::timer_service;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};
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
    timer_service::get_active_timer(db)
}

fn pause_timer_internal(db: &DbConnection, timer: &ActiveTimer) -> crate::error::Result<()> {
    let conn = db.lock();
    timer_service::pause_running_timer_at(&conn, timer, get_timestamp())
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
pub fn auto_pause_if_running(app_handle: &AppHandle, db: &DbConnection, reason: AutoPauseReason) {
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
                        eprintln!("[Auto-Pause] Failed to emit event to frontend: {}", e);
                    } else {
                        println!(
                            "[Auto-Pause] Timer successfully paused. Reason: {:?}",
                            reason
                        );
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

pub static SCREEN_LOCK_TIME: AtomicI64 = AtomicI64::new(0);
pub static IS_LOCKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Handle system away started event (screen lock or display off)
pub fn handle_away_started(app_handle: &AppHandle, db: &DbConnection) {
    let now = get_timestamp();
    // Only store if not already tracking away time
    if SCREEN_LOCK_TIME
        .compare_exchange(0, now, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        println!("[System Events] User away started at {}", now);

        // Auto-pause if running
        auto_pause_if_running(app_handle, db, AutoPauseReason::ScreenLock);
    }
}

/// Handle system away ended event (screen unlock or display on)
pub fn handle_away_ended(app_handle: &AppHandle, db: &DbConnection) {
    let now = get_timestamp();
    let lock_time = SCREEN_LOCK_TIME.swap(0, Ordering::SeqCst);

    if lock_time > 0 {
        let away_seconds = now - lock_time;
        println!(
            "[System Events] User returned. Away for {} seconds",
            away_seconds
        );

        // Define a reasonable minimum break time to show the resume window
        if away_seconds > 10 {
            let (task_id, task_title) = match get_active_timer_internal(db) {
                Ok(Some(timer)) => (
                    Some(timer.task_id),
                    timer.task_title.unwrap_or_else(|| "".to_string()),
                ),
                _ => (None, "".to_string()),
            };

            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = crate::commands::window::open_resume_window(
                    app_handle_clone,
                    task_id,
                    task_title,
                    away_seconds,
                    None,
                )
                .await
                {
                    eprintln!("Failed to open resume window: {}", e);
                }
            });
        }
    }
}
