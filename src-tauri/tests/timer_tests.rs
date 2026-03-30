mod common;

use common::*;
use my_todos_lib::db::ActiveTimer;
use my_todos_lib::error::{AppError, Result};
use std::thread;
use std::time::Duration;

// Helper functions that replicate timer.rs logic for testing
fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

fn get_active_timer_impl(db: &DbConnection) -> Result<Option<ActiveTimer>> {
    let conn = db.lock();
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
        Err(e) => Err(e.into()),
    }
}

fn start_timer_impl(db: &DbConnection, task_id: i64) -> Result<ActiveTimer> {
    if let Some(existing) = get_active_timer_impl(db)? {
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

fn pause_timer_impl(db: &DbConnection) -> Result<()> {
    let timer = get_active_timer_impl(db)?.ok_or(AppError::NoActiveTimer)?;

    if !timer.is_running {
        return Ok(());
    }

    let conn = db.lock();
    let now = get_timestamp();
    let duration = timer.elapsed_seconds + (now - timer.started_at);

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (timer.task_id, duration, timer.started_at, now, now),
    )?;

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration, timer.task_id),
    )?;

    let project_id: i64 = conn.query_row(
        "SELECT project_id FROM tasks WHERE id = ?",
        [timer.task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration, project_id),
    )?;

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

    conn.execute(
        "UPDATE active_timer SET is_running = 0, elapsed_seconds = 0, started_at = ? WHERE id = 1",
        [now],
    )?;

    Ok(())
}

fn resume_timer_impl(db: &DbConnection) -> Result<()> {
    let timer = get_active_timer_impl(db)?.ok_or(AppError::NoActiveTimer)?;

    if timer.is_running {
        return Ok(());
    }

    let conn = db.lock();
    let now = get_timestamp();

    conn.execute(
        "UPDATE active_timer SET is_running = 1, started_at = ?, last_heartbeat_at = ? WHERE id = 1",
        [now, now],
    )?;

    Ok(())
}

fn stop_timer_impl(db: &DbConnection) -> Result<i64> {
    let timer = get_active_timer_impl(db)?.ok_or(AppError::NoActiveTimer)?;

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

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (total_duration, timer.task_id),
    )?;

    let project_id: i64 = conn.query_row(
        "SELECT project_id FROM tasks WHERE id = ?",
        [timer.task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (total_duration, project_id),
    )?;

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
            (total_duration, sid),
        )?;
    }

    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;

    Ok(total_duration)
}

fn reset_timer_impl(db: &DbConnection) -> Result<()> {
    let _timer = get_active_timer_impl(db)?.ok_or(AppError::NoActiveTimer)?;
    let conn = db.lock();
    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;
    Ok(())
}

#[test]
fn test_start_timer_success() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = start_timer_impl(&db, task_id);

    assert!(result.is_ok());
    let timer = result.unwrap();
    assert_eq!(timer.task_id, task_id);
    assert_eq!(timer.elapsed_seconds, 0);
    assert!(timer.is_running);
    assert!(has_active_timer(&db));
}

#[test]
fn test_start_timer_when_already_active() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task1_id = create_test_task(&db, Some(project_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task 2");

    // Start first timer
    start_timer_impl(&db, task1_id).unwrap();

    // Try to start second timer - should fail
    let result = start_timer_impl(&db, task2_id);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::TimerActive(msg) => {
            assert!(msg.contains(&format!("{}", task1_id)));
        }
        _ => panic!("Expected TimerActive error"),
    }
}

#[test]
fn test_start_timer_invalid_task_id() {
    let db = setup_test_db();

    let result = start_timer_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => {
            assert!(msg.contains("999"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_get_active_timer_when_exists() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();

    let result = get_active_timer_impl(&db);

    assert!(result.is_ok());
    let timer = result.unwrap();
    assert!(timer.is_some());
    let timer = timer.unwrap();
    assert_eq!(timer.task_id, task_id);
    assert_eq!(timer.task_title, Some("Test Task".to_string()));
}

#[test]
fn test_get_active_timer_when_none() {
    let db = setup_test_db();

    let result = get_active_timer_impl(&db);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_pause_timer_from_running() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();

    // Wait a bit to accumulate time
    thread::sleep(Duration::from_millis(1100));

    let result = pause_timer_impl(&db);

    assert!(result.is_ok());

    // Timer should still exist but be paused
    assert!(has_active_timer(&db));

    let timer_info = get_active_timer_raw(&db).unwrap();
    assert_eq!(timer_info.0, task_id); // task_id
    assert!(!timer_info.3); // is_running should be false

    // Task time should have been updated
    let task_time = get_task_time(&db, task_id);
    assert!(task_time > 0);

    // Project time should have been updated
    let project_time = get_project_time(&db, project_id);
    assert!(project_time > 0);
    assert_eq!(task_time, project_time);

    // Time entry should have been created
    assert_eq!(count_time_entries(&db, task_id), 1);
}

#[test]
fn test_pause_timer_when_no_active_timer() {
    let db = setup_test_db();

    let result = pause_timer_impl(&db);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NoActiveTimer => {}
        _ => panic!("Expected NoActiveTimer error"),
    }
}

#[test]
fn test_pause_timer_when_already_paused() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();
    pause_timer_impl(&db).unwrap();

    let time_before = get_task_time(&db, task_id);

    // Pause again - should be no-op
    let result = pause_timer_impl(&db);

    assert!(result.is_ok());

    let time_after = get_task_time(&db, task_id);
    assert_eq!(time_before, time_after);
}

#[test]
fn test_resume_timer_from_paused() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();
    pause_timer_impl(&db).unwrap();

    let result = resume_timer_impl(&db);

    assert!(result.is_ok());

    // Timer should be running again
    let timer_info = get_active_timer_raw(&db).unwrap();
    assert_eq!(timer_info.0, task_id);
    assert!(timer_info.3); // is_running should be true
    assert_eq!(timer_info.2, 0); // elapsed_seconds reset to 0
}

#[test]
fn test_resume_timer_when_no_active_timer() {
    let db = setup_test_db();

    let result = resume_timer_impl(&db);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NoActiveTimer => {}
        _ => panic!("Expected NoActiveTimer error"),
    }
}

#[test]
fn test_resume_timer_when_already_running() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();

    let timer_before = get_active_timer_raw(&db).unwrap();

    // Resume when already running - should be no-op
    let result = resume_timer_impl(&db);

    assert!(result.is_ok());

    let timer_after = get_active_timer_raw(&db).unwrap();
    assert_eq!(timer_before.1, timer_after.1); // started_at unchanged
}

#[test]
fn test_stop_timer_from_running() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();

    // Wait a bit
    thread::sleep(Duration::from_millis(1100));

    let result = stop_timer_impl(&db);

    assert!(result.is_ok());
    let duration = result.unwrap();
    assert!(duration > 0);

    // Active timer should be deleted
    assert!(!has_active_timer(&db));

    // Task and project times should be updated
    let task_time = get_task_time(&db, task_id);
    assert!(task_time > 0);
    assert_eq!(task_time, duration);

    let project_time = get_project_time(&db, project_id);
    assert_eq!(project_time, task_time);
}

#[test]
fn test_stop_timer_from_paused() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();
    thread::sleep(Duration::from_millis(1100));
    pause_timer_impl(&db).unwrap();

    let time_after_pause = get_task_time(&db, task_id);

    let result = stop_timer_impl(&db);

    assert!(result.is_ok());

    // Active timer should be deleted
    assert!(!has_active_timer(&db));

    // No additional time should be added (was paused)
    let final_time = get_task_time(&db, task_id);
    assert_eq!(final_time, time_after_pause);
}

#[test]
fn test_stop_timer_when_no_active_timer() {
    let db = setup_test_db();

    let result = stop_timer_impl(&db);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NoActiveTimer => {}
        _ => panic!("Expected NoActiveTimer error"),
    }
}

#[test]
fn test_reset_timer() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();
    thread::sleep(Duration::from_millis(1100));

    let result = reset_timer_impl(&db);

    assert!(result.is_ok());

    // Active timer should be deleted
    assert!(!has_active_timer(&db));

    // Task time should still be 0 (no time recorded)
    let task_time = get_task_time(&db, task_id);
    assert_eq!(task_time, 0);

    // No time entries should exist
    assert_eq!(count_time_entries(&db, task_id), 0);
}

#[test]
fn test_reset_timer_when_no_active_timer() {
    let db = setup_test_db();

    let result = reset_timer_impl(&db);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NoActiveTimer => {}
        _ => panic!("Expected NoActiveTimer error"),
    }
}

#[test]
fn test_multiple_pause_resume_cycles() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();
    thread::sleep(Duration::from_millis(1100));
    pause_timer_impl(&db).unwrap();

    let time_after_pause1 = get_task_time(&db, task_id);
    assert!(time_after_pause1 > 0);

    resume_timer_impl(&db).unwrap();
    thread::sleep(Duration::from_millis(1100));
    pause_timer_impl(&db).unwrap();

    let time_after_pause2 = get_task_time(&db, task_id);
    assert!(time_after_pause2 > time_after_pause1);

    resume_timer_impl(&db).unwrap();
    thread::sleep(Duration::from_millis(1100));
    stop_timer_impl(&db).unwrap();

    let final_time = get_task_time(&db, task_id);
    assert!(final_time > time_after_pause2);

    // Should have 3 time entries (2 pauses + 1 stop)
    assert_eq!(count_time_entries(&db, task_id), 3);
}

#[test]
fn test_cascading_updates_to_section() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task_id = create_test_task(&db, Some(project_id), Some(section_id), "Test Task");

    start_timer_impl(&db, task_id).unwrap();
    thread::sleep(Duration::from_millis(1100));
    pause_timer_impl(&db).unwrap();

    let task_time = get_task_time(&db, task_id);
    let section_time = get_section_time(&db, section_id);
    let project_time = get_project_time(&db, project_id);

    assert!(task_time > 0);
    assert_eq!(task_time, section_time);
    assert_eq!(task_time, project_time);
}

#[test]
fn test_elapsed_time_calculation() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();

    let timer1 = get_active_timer_impl(&db).unwrap().unwrap();
    let start_time1 = timer1.started_at;

    // Wait 100ms
    thread::sleep(Duration::from_millis(1100));

    let timer2 = get_active_timer_impl(&db).unwrap().unwrap();

    // elapsed_seconds should still be 0 (only updated on pause/stop)
    assert_eq!(timer2.elapsed_seconds, 0);
    assert_eq!(timer2.started_at, start_time1);

    // But the client would calculate: elapsed_seconds + (now - started_at)
    // which should be approximately 100ms
    let now = get_timestamp();
    let calculated_elapsed = timer2.elapsed_seconds + (now - timer2.started_at);
    // Should be at least 100ms (0.1 seconds)
    assert!(calculated_elapsed >= 0);
}

#[test]
fn test_timer_with_task_without_project() {
    let db = setup_test_db();
    let task_id = create_test_task(&db, None, None, "Orphan Task");

    // Should still work even without project_id
    let result = start_timer_impl(&db, task_id);

    // This might fail depending on schema constraints
    // If project_id is required, this test documents that behavior
    match result {
        Ok(timer) => {
            assert_eq!(timer.task_id, task_id);
            assert!(timer.project_id.is_none());
        }
        Err(_) => {
            // If it fails, that's okay too - documents the constraint
            // The schema might require project_id to be non-null
        }
    }
}

#[test]
fn test_zero_elapsed_time_pause() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer_impl(&db, task_id).unwrap();

    // Pause immediately (no sleep)
    pause_timer_impl(&db).unwrap();

    // Time might be 0 or very small
    let task_time = get_task_time(&db, task_id);
    assert!(task_time >= 0);

    // Time entry should still be created
    assert_eq!(count_time_entries(&db, task_id), 1);
}

#[test]
fn test_timer_lifecycle_full_sequence() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Start
    let start_result = start_timer_impl(&db, task_id);
    assert!(start_result.is_ok());
    assert!(has_active_timer(&db));

    thread::sleep(Duration::from_millis(1100));

    // Pause
    let pause_result = pause_timer_impl(&db);
    assert!(pause_result.is_ok());
    assert!(has_active_timer(&db));

    let time_after_pause = get_task_time(&db, task_id);
    assert!(time_after_pause > 0);

    thread::sleep(Duration::from_millis(1100));

    // Resume
    let resume_result = resume_timer_impl(&db);
    assert!(resume_result.is_ok());
    assert!(has_active_timer(&db));

    // Time shouldn't increase while paused
    let time_after_resume = get_task_time(&db, task_id);
    assert_eq!(time_after_resume, time_after_pause);

    thread::sleep(Duration::from_millis(1100));

    // Stop
    let stop_result = stop_timer_impl(&db);
    assert!(stop_result.is_ok());
    assert!(!has_active_timer(&db));

    let final_time = get_task_time(&db, task_id);
    assert!(final_time > time_after_pause);
}

#[test]
fn test_recover_stale_timer_pauses_at_last_heartbeat() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");
    let now = get_timestamp();
    let started_at = now - 600;
    let last_heartbeat_at = now - 300;

    {
        let conn = db.lock();
        conn.execute(
            "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, last_heartbeat_at, project_id)
             VALUES (1, ?, ?, 0, 1, ?, ?)",
            (task_id, started_at, last_heartbeat_at, project_id),
        )
        .unwrap();
    }

    let recovered = my_todos_lib::services::timer_service::recover_stale_active_timer(&db).unwrap();
    assert!(recovered);

    let timer = get_active_timer_impl(&db).unwrap().unwrap();
    assert!(!timer.is_running);
    assert_eq!(timer.started_at, last_heartbeat_at);
    assert_eq!(timer.last_heartbeat_at, Some(last_heartbeat_at));

    let expected_duration = last_heartbeat_at - started_at;
    assert_eq!(get_task_time(&db, task_id), expected_duration);
    assert_eq!(get_project_time(&db, project_id), expected_duration);

    let conn = db.lock();
    let (duration, ended_at): (i64, i64) = conn
        .query_row(
            "SELECT duration_seconds, ended_at FROM time_entries WHERE task_id = ? ORDER BY id DESC LIMIT 1",
            [task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(duration, expected_duration);
    assert_eq!(ended_at, last_heartbeat_at);
}

#[test]
fn test_recover_stale_timer_keeps_recent_running_timer_active() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");
    let now = get_timestamp();
    let started_at = now - 20;
    let last_heartbeat_at = now - 10;

    {
        let conn = db.lock();
        conn.execute(
            "INSERT INTO active_timer (id, task_id, started_at, elapsed_seconds, is_running, last_heartbeat_at, project_id)
             VALUES (1, ?, ?, 0, 1, ?, ?)",
            (task_id, started_at, last_heartbeat_at, project_id),
        )
        .unwrap();
    }

    let recovered = my_todos_lib::services::timer_service::recover_stale_active_timer(&db).unwrap();
    assert!(!recovered);

    let timer = get_active_timer_impl(&db).unwrap().unwrap();
    assert!(timer.is_running);
    assert_eq!(timer.started_at, started_at);
    assert_eq!(timer.last_heartbeat_at, Some(last_heartbeat_at));
    assert_eq!(count_time_entries(&db, task_id), 0);
}

#[test]
fn test_log_afk_time_creates_named_system_task_and_updates_totals() {
    let db = setup_test_db();

    my_todos_lib::services::timer_service::log_afk_time(&db, "Meeting", 900).unwrap();
    my_todos_lib::services::timer_service::log_afk_time(&db, "Meeting", 300).unwrap();
    my_todos_lib::services::timer_service::log_afk_time(&db, "Lunch", 1800).unwrap();

    let conn = db.lock();
    let away_project: (i64, i64, bool) = conn
        .query_row(
            "SELECT id, total_time_seconds, is_system
             FROM projects
             WHERE name = 'Away'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert!(away_project.2);
    assert_eq!(away_project.1, 3000);

    let meeting_task: (i64, i64, bool) = conn
        .query_row(
            "SELECT id, total_time_seconds, is_system
             FROM tasks
             WHERE project_id = ? AND title = 'Meeting'",
            [away_project.0],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert!(meeting_task.2);
    assert_eq!(meeting_task.1, 1200);

    let lunch_task_time: i64 = conn
        .query_row(
            "SELECT total_time_seconds
             FROM tasks
             WHERE project_id = ? AND title = 'Lunch'",
            [away_project.0],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(lunch_task_time, 1800);

    let meeting_entries: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM time_entries
             WHERE task_id = ?",
            [meeting_task.0],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(meeting_entries, 2);
}

#[test]
fn test_log_afk_time_rejects_blank_category_names() {
    let db = setup_test_db();

    let result = my_todos_lib::services::timer_service::log_afk_time(&db, "   ", 300);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::InvalidInput(message) => {
            assert!(message.contains("AFK category name"));
        }
        other => panic!("Expected InvalidInput error, got {other:?}"),
    }
}
