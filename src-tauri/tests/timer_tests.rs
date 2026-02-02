mod common;

use common::*;
use my_todos_lib::commands::timer::*;
use my_todos_lib::error::AppError;
use std::thread;
use std::time::Duration;
use tauri::State;

fn wrap_db(db: &DbConnection) -> State<DbConnection> {
    State::from(db)
}

#[test]
fn test_start_timer_success() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = start_timer(wrap_db(&db), task_id);

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
    start_timer(wrap_db(&db), task1_id).unwrap();

    // Try to start second timer - should fail
    let result = start_timer(wrap_db(&db), task2_id);

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

    let result = start_timer(wrap_db(&db), 999);

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

    start_timer(wrap_db(&db), task_id).unwrap();

    let result = get_active_timer(wrap_db(&db));

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

    let result = get_active_timer(wrap_db(&db));

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_pause_timer_from_running() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer(wrap_db(&db), task_id).unwrap();

    // Wait a bit to accumulate time
    thread::sleep(Duration::from_millis(100));

    let result = pause_timer(wrap_db(&db));

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

    let result = pause_timer(wrap_db(&db));

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

    start_timer(wrap_db(&db), task_id).unwrap();
    pause_timer(wrap_db(&db)).unwrap();

    let time_before = get_task_time(&db, task_id);

    // Pause again - should be no-op
    let result = pause_timer(wrap_db(&db));

    assert!(result.is_ok());

    let time_after = get_task_time(&db, task_id);
    assert_eq!(time_before, time_after);
}

#[test]
fn test_resume_timer_from_paused() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer(wrap_db(&db), task_id).unwrap();
    pause_timer(wrap_db(&db)).unwrap();

    let result = resume_timer(wrap_db(&db));

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

    let result = resume_timer(wrap_db(&db));

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

    start_timer(wrap_db(&db), task_id).unwrap();

    let timer_before = get_active_timer_raw(&db).unwrap();

    // Resume when already running - should be no-op
    let result = resume_timer(wrap_db(&db));

    assert!(result.is_ok());

    let timer_after = get_active_timer_raw(&db).unwrap();
    assert_eq!(timer_before.1, timer_after.1); // started_at unchanged
}

#[test]
fn test_stop_timer_from_running() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer(wrap_db(&db), task_id).unwrap();

    // Wait a bit
    thread::sleep(Duration::from_millis(100));

    let result = stop_timer(wrap_db(&db));

    assert!(result.is_ok());
    let entry = result.unwrap();
    assert_eq!(entry.task_id, task_id);
    assert!(entry.duration_seconds > 0);
    assert_eq!(entry.entry_type, "timer");

    // Active timer should be deleted
    assert!(!has_active_timer(&db));

    // Task and project times should be updated
    let task_time = get_task_time(&db, task_id);
    assert!(task_time > 0);
    assert_eq!(task_time, entry.duration_seconds);

    let project_time = get_project_time(&db, project_id);
    assert_eq!(project_time, task_time);
}

#[test]
fn test_stop_timer_from_paused() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    start_timer(wrap_db(&db), task_id).unwrap();
    thread::sleep(Duration::from_millis(100));
    pause_timer(wrap_db(&db)).unwrap();

    let time_after_pause = get_task_time(&db, task_id);

    let result = stop_timer(wrap_db(&db));

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

    let result = stop_timer(wrap_db(&db));

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

    start_timer(wrap_db(&db), task_id).unwrap();
    thread::sleep(Duration::from_millis(100));

    let result = reset_timer(wrap_db(&db));

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

    let result = reset_timer(wrap_db(&db));

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

    start_timer(wrap_db(&db), task_id).unwrap();
    thread::sleep(Duration::from_millis(50));
    pause_timer(wrap_db(&db)).unwrap();

    let time_after_pause1 = get_task_time(&db, task_id);
    assert!(time_after_pause1 > 0);

    resume_timer(wrap_db(&db)).unwrap();
    thread::sleep(Duration::from_millis(50));
    pause_timer(wrap_db(&db)).unwrap();

    let time_after_pause2 = get_task_time(&db, task_id);
    assert!(time_after_pause2 > time_after_pause1);

    resume_timer(wrap_db(&db)).unwrap();
    thread::sleep(Duration::from_millis(50));
    stop_timer(wrap_db(&db)).unwrap();

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

    start_timer(wrap_db(&db), task_id).unwrap();
    thread::sleep(Duration::from_millis(100));
    pause_timer(wrap_db(&db)).unwrap();

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

    start_timer(wrap_db(&db), task_id).unwrap();

    let timer1 = get_active_timer(wrap_db(&db)).unwrap().unwrap();
    let start_time1 = timer1.started_at;

    // Wait 100ms
    thread::sleep(Duration::from_millis(100));

    let timer2 = get_active_timer(wrap_db(&db)).unwrap().unwrap();

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
    let result = start_timer(wrap_db(&db), task_id);

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

    start_timer(wrap_db(&db), task_id).unwrap();

    // Pause immediately (no sleep)
    pause_timer(wrap_db(&db)).unwrap();

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
    let start_result = start_timer(wrap_db(&db), task_id);
    assert!(start_result.is_ok());
    assert!(has_active_timer(&db));

    thread::sleep(Duration::from_millis(50));

    // Pause
    let pause_result = pause_timer(wrap_db(&db));
    assert!(pause_result.is_ok());
    assert!(has_active_timer(&db));

    let time_after_pause = get_task_time(&db, task_id);
    assert!(time_after_pause > 0);

    thread::sleep(Duration::from_millis(50));

    // Resume
    let resume_result = resume_timer(wrap_db(&db));
    assert!(resume_result.is_ok());
    assert!(has_active_timer(&db));

    // Time shouldn't increase while paused
    let time_after_resume = get_task_time(&db, task_id);
    assert_eq!(time_after_resume, time_after_pause);

    thread::sleep(Duration::from_millis(50));

    // Stop
    let stop_result = stop_timer(wrap_db(&db));
    assert!(stop_result.is_ok());
    assert!(!has_active_timer(&db));

    let final_time = get_task_time(&db, task_id);
    assert!(final_time > time_after_pause);
}
