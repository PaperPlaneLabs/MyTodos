mod common;

use common::{
    count_time_entries, create_test_project, create_test_task, get_project_time, get_task_time,
    has_active_timer, setup_test_db,
};
use my_todos_lib::services::timer_service;

#[test]
fn start_timed_timer_persists_bounded_session_state() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Focus");
    let task_id = create_test_task(&db, Some(project_id), None, "Deep work");

    let timer = timer_service::start_timed_timer(&db, task_id, 25 * 60).unwrap();

    assert_eq!(timer.timer_limit_seconds, Some(25 * 60));
    assert_eq!(timer.timer_remaining_seconds, Some(25 * 60));
    assert_eq!(timer.timer_expires_at, Some(timer.started_at + 25 * 60));
    assert!(timer.is_running);
}

#[test]
fn timed_timer_rejects_invalid_durations() {
    let db = setup_test_db();
    let task_id = create_test_task(&db, None, None, "Focus");

    assert!(timer_service::start_timed_timer(&db, task_id, 0).is_err());
    assert!(timer_service::start_timed_timer(&db, task_id, 24 * 60 * 60 + 1).is_err());
    assert!(!has_active_timer(&db));

    {
        let conn = db.lock();
        conn.execute("UPDATE tasks SET completed = 1 WHERE id = ?", [task_id])
            .unwrap();
    }
    assert!(timer_service::start_timed_timer(&db, task_id, 60).is_err());
    assert!(!has_active_timer(&db));
}

#[test]
fn pause_and_resume_freeze_and_restore_timed_countdown() {
    let db = setup_test_db();
    let task_id = create_test_task(&db, None, None, "Focus");
    timer_service::start_timed_timer(&db, task_id, 120).unwrap();

    let simulated_now = chrono::Utc::now().timestamp();
    {
        let conn = db.lock();
        conn.execute(
            "UPDATE active_timer
             SET started_at = ?, last_heartbeat_at = ?, timer_expires_at = ?
             WHERE id = 1",
            (simulated_now - 30, simulated_now, simulated_now + 90),
        )
        .unwrap();
    }

    timer_service::pause_timer(&db).unwrap();
    let paused = timer_service::get_active_timer(&db).unwrap().unwrap();
    let paused_remaining = paused.timer_remaining_seconds.unwrap();
    assert!((89..=90).contains(&paused_remaining));
    assert_eq!(paused.timer_expires_at, None);
    assert!(!paused.is_running);
    assert_eq!(count_time_entries(&db, task_id), 1);

    timer_service::resume_timer(&db).unwrap();
    let resumed = timer_service::get_active_timer(&db).unwrap().unwrap();
    assert!(resumed.is_running);
    assert_eq!(
        resumed.timer_expires_at,
        Some(resumed.started_at + paused_remaining)
    );
}

#[test]
fn expiry_caps_time_at_deadline_and_is_idempotent() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Focus");
    let task_id = create_test_task(&db, Some(project_id), None, "Deep work");
    timer_service::start_timed_timer(&db, task_id, 60).unwrap();

    {
        let conn = db.lock();
        conn.execute(
            "UPDATE active_timer
             SET started_at = 1000, last_heartbeat_at = 1060,
                 timer_remaining_seconds = 60, timer_expires_at = 1060
             WHERE id = 1",
            [],
        )
        .unwrap();
    }

    let completion = timer_service::finish_expired_timed_timer_at(&db, 1200)
        .unwrap()
        .unwrap();

    assert_eq!(completion.task_id, task_id);
    assert_eq!(completion.task_title, "Deep work");
    assert_eq!(completion.duration_seconds, 60);
    assert_eq!(completion.finished_at, 1060);
    assert_eq!(count_time_entries(&db, task_id), 1);
    assert_eq!(get_task_time(&db, task_id), 60);
    assert_eq!(get_project_time(&db, project_id), 60);
    assert!(!has_active_timer(&db));

    assert_eq!(
        timer_service::finish_expired_timed_timer_at(&db, 1300).unwrap(),
        None
    );
    assert_eq!(count_time_entries(&db, task_id), 1);
}

#[test]
fn expiry_does_not_finish_a_timer_before_its_deadline() {
    let db = setup_test_db();
    let task_id = create_test_task(&db, None, None, "Focus");
    let timer = timer_service::start_timed_timer(&db, task_id, 300).unwrap();

    let result =
        timer_service::finish_expired_timed_timer_at(&db, timer.timer_expires_at.unwrap() - 1)
            .unwrap();

    assert_eq!(result, None);
    assert!(has_active_timer(&db));
    assert_eq!(count_time_entries(&db, task_id), 0);
}

#[test]
fn expiry_adds_only_the_remaining_segment_after_a_pause() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Focus");
    let task_id = create_test_task(&db, Some(project_id), None, "Deep work");
    timer_service::start_timed_timer(&db, task_id, 120).unwrap();

    {
        let conn = db.lock();
        conn.execute(
            "INSERT INTO time_entries (
                 task_id, entry_type, duration_seconds, started_at, ended_at, created_at
             ) VALUES (?, 'timer', 40, 900, 940, 940)",
            [task_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE tasks SET total_time_seconds = 40 WHERE id = ?",
            [task_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE projects SET total_time_seconds = 40 WHERE id = ?",
            [project_id],
        )
        .unwrap();
        conn.execute(
            "UPDATE active_timer
             SET started_at = 1000, timer_remaining_seconds = 80,
                 timer_expires_at = 1080, last_heartbeat_at = 1080
             WHERE id = 1",
            [],
        )
        .unwrap();
    }

    timer_service::finish_expired_timed_timer_at(&db, 1200)
        .unwrap()
        .unwrap();

    assert_eq!(count_time_entries(&db, task_id), 2);
    assert_eq!(get_task_time(&db, task_id), 120);
    assert_eq!(get_project_time(&db, project_id), 120);
}

#[test]
fn stale_recovery_pauses_at_heartbeat_instead_of_counting_offline_time() {
    let db = setup_test_db();
    let task_id = create_test_task(&db, None, None, "Focus");
    timer_service::start_timed_timer(&db, task_id, 300).unwrap();
    let now = chrono::Utc::now().timestamp();

    {
        let conn = db.lock();
        conn.execute(
            "UPDATE active_timer
             SET started_at = ?, last_heartbeat_at = ?, timer_expires_at = ?
             WHERE id = 1",
            (now - 300, now - 200, now),
        )
        .unwrap();
    }

    assert!(timer_service::recover_stale_active_timer(&db).unwrap());
    let recovered = timer_service::get_active_timer(&db).unwrap().unwrap();
    assert!(!recovered.is_running);
    assert_eq!(recovered.timer_remaining_seconds, Some(200));
    assert_eq!(recovered.timer_expires_at, None);
    assert_eq!(get_task_time(&db, task_id), 100);
}
