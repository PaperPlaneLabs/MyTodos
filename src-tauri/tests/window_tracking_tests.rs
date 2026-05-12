mod common;

use common::{create_test_project, create_test_task, get_timestamp, setup_test_db};
use my_todos_lib::services::{timer_service, window_tracking_service};

fn insert_window_activity(db: &common::DbConnection, identifier: &str, name: &str, duration: i64) {
    let now = get_timestamp();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO window_activity_entries (
            app_identifier,
            app_name,
            started_at,
            ended_at,
            duration_seconds,
            created_at
         ) VALUES (?, ?, ?, ?, ?, ?)",
        (identifier, name, now - duration, now, duration, now),
    )
    .unwrap();
}

fn insert_afk_project_no_entries(db: &common::DbConnection, title: &str) {
    let now = get_timestamp();
    let conn = db.lock();
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
         ) VALUES ('Away', 'Automatically tracked away-from-keyboard time', '#f59e0b', 0, 0, 1, ?, ?)",
        (now, now),
    )
    .unwrap();
    let project_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO tasks (
            project_id,
            title,
            description,
            total_time_seconds,
            is_system,
            created_at,
            updated_at
         ) VALUES (?, ?, 'Unused AFK category', 0, 1, ?, ?)",
        (project_id, title, now, now),
    )
    .unwrap();
}

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

#[test]
fn test_window_tracking_stats_include_logged_afk_categories() {
    let db = setup_test_db();
    insert_window_activity(&db, "code", "Visual Studio Code", 300);
    timer_service::log_afk_time(&db, "Meeting", 900).unwrap();
    timer_service::log_afk_time(&db, "Lunch", 300).unwrap();

    let stats = window_tracking_service::get_stats(&db).unwrap();

    let code = stats
        .today_apps
        .iter()
        .find(|entry| entry.app_identifier == "code")
        .expect("App activity should be present in today's chart");
    assert!(matches!(
        &code.kind,
        window_tracking_service::ActivityEntryKind::App
    ));
    assert_eq!(code.total_seconds, 300);

    let meeting = stats
        .today_apps
        .iter()
        .find(|entry| entry.app_name == "Meeting")
        .expect("Logged AFK category should be present in today's chart");
    assert!(matches!(
        &meeting.kind,
        window_tracking_service::ActivityEntryKind::Afk
    ));
    assert!(meeting.app_identifier.starts_with("afk:"));
    assert_eq!(meeting.total_seconds, 900);

    let pie_meeting = stats
        .apps
        .iter()
        .find(|entry| entry.app_name == "Meeting")
        .expect("Logged AFK category should be present in pie chart data");
    assert_eq!(pie_meeting.total_seconds, 900);

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_total = stats
        .week_daily
        .iter()
        .find(|entry| entry.date == today)
        .expect("Today's weekly aggregate should be present")
        .total_seconds;
    assert_eq!(today_total, 1500);
}

#[test]
fn test_window_tracking_stats_include_default_break_away_time() {
    let db = setup_test_db();
    insert_window_activity(&db, "code", "Visual Studio Code", 300);
    timer_service::log_break_time(&db, 600).unwrap();

    let stats = window_tracking_service::get_stats(&db).unwrap();

    let break_time = stats
        .today_apps
        .iter()
        .find(|entry| entry.app_name == "Break")
        .expect("Default resume-window break time should be visible");
    assert!(matches!(
        &break_time.kind,
        window_tracking_service::ActivityEntryKind::Afk
    ));
    assert!(break_time.app_identifier.starts_with("afk:"));
    assert_eq!(break_time.total_seconds, 600);

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_total = stats
        .week_daily
        .iter()
        .find(|entry| entry.date == today)
        .expect("Today's weekly aggregate should be present")
        .total_seconds;
    assert_eq!(today_total, 900);
}

#[test]
fn test_window_tracking_stats_skip_unused_afk_categories() {
    let db = setup_test_db();
    insert_afk_project_no_entries(&db, "Snack");
    insert_window_activity(&db, "excel", "Excel", 600);

    let stats = window_tracking_service::get_stats(&db).unwrap();

    assert!(stats
        .today_apps
        .iter()
        .all(|entry| !entry.app_identifier.starts_with("afk:")));
    assert!(stats.apps.iter().all(|entry| entry.app_name != "Snack"));
    assert_eq!(stats.today_apps.len(), 1);
    assert_eq!(stats.today_apps[0].app_identifier, "excel");
    assert!(matches!(
        &stats.today_apps[0].kind,
        window_tracking_service::ActivityEntryKind::App
    ));
}
