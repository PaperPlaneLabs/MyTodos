mod common;

use common::{create_test_project, create_test_task, get_timestamp, setup_test_db};
use my_todos_lib::services::{timer_service, window_tracking_service};

#[test]
fn test_window_tracking_setting_blocks_task_timer_start() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Work");
    let task_id = create_test_task(&db, Some(project_id), None, "Task");

    let settings = window_tracking_service::set_enabled(&db, true).unwrap();
    assert!(settings.enabled);

    let result = timer_service::start_timer(&db, task_id);
    assert!(result.is_err());

    window_tracking_service::set_enabled(&db, false).unwrap();
    let timer = timer_service::start_timer(&db, task_id).unwrap();
    assert_eq!(timer.task_id, task_id);
}

#[test]
fn test_window_tracking_records_segments_on_app_change() {
    let db = setup_test_db();
    window_tracking_service::set_enabled(&db, true).unwrap();

    window_tracking_service::record_foreground_app(
        &db,
        window_tracking_service::ForegroundApp {
            app_identifier: "code".to_string(),
            app_name: "Visual Studio Code".to_string(),
        },
    )
    .unwrap();

    let started_at = get_timestamp() - 120;
    {
        let conn = db.lock();
        conn.execute(
            "UPDATE active_window_tracking
             SET app_started_at = ?, work_started_at = ?, last_seen_at = ?
             WHERE id = 1",
            (started_at, started_at, started_at),
        )
        .unwrap();
    }

    window_tracking_service::record_foreground_app(
        &db,
        window_tracking_service::ForegroundApp {
            app_identifier: "chrome".to_string(),
            app_name: "Google Chrome".to_string(),
        },
    )
    .unwrap();

    let stats = window_tracking_service::get_stats(&db).unwrap();
    let code = stats
        .today_apps
        .iter()
        .find(|entry| entry.app_identifier == "code")
        .expect("Code segment should be present");

    assert!(code.total_seconds >= 100);

    let state = window_tracking_service::get_state(&db).unwrap();
    let active = state.active.expect("Chrome should be active");
    assert_eq!(active.app_identifier, "chrome");
    assert_eq!(active.work_started_at, started_at);
}

#[test]
fn test_window_tracking_pause_closes_active_segment_without_disabling() {
    let db = setup_test_db();
    window_tracking_service::set_enabled(&db, true).unwrap();

    window_tracking_service::record_foreground_app(
        &db,
        window_tracking_service::ForegroundApp {
            app_identifier: "excel".to_string(),
            app_name: "Excel".to_string(),
        },
    )
    .unwrap();

    let started_at = get_timestamp() - 60;
    {
        let conn = db.lock();
        conn.execute(
            "UPDATE active_window_tracking
             SET app_started_at = ?, work_started_at = ?, last_seen_at = ?
             WHERE id = 1",
            (started_at, started_at, started_at),
        )
        .unwrap();
    }

    let state = window_tracking_service::set_paused(&db, true).unwrap();
    assert!(state.enabled);
    assert!(state.paused);
    assert!(state.active.is_none());

    let stats = window_tracking_service::get_stats(&db).unwrap();
    let excel = stats
        .today_apps
        .iter()
        .find(|entry| entry.app_identifier == "excel")
        .expect("Excel segment should be present");
    assert!(excel.total_seconds >= 50);
}
