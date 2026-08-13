mod common;

use chrono::NaiveDate;
use common::*;
use my_todos_lib::services::calendar_service::{
    self, CalendarEventInput, RecurrenceFrequency, RecurrenceRule,
};

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn input(title: &str, start_date: &str, end_date: &str) -> CalendarEventInput {
    CalendarEventInput {
        title: title.into(),
        description: None,
        is_all_day: true,
        color: Some("#6366f1".into()),
        start_date: start_date.into(),
        end_date: end_date.into(),
        start_time: None,
        end_time: None,
        start_at: None,
        end_at: None,
        timezone: Some("Asia/Kolkata".into()),
        recurrence_rule: None,
    }
}

#[test]
fn migrates_legacy_date_only_events_to_end_exclusive_ranges() {
    let db = setup_test_db();
    let conn = db.lock();
    conn.execute(
        "INSERT INTO calendar_events (title, date, is_all_day, created_at)
         VALUES ('Legacy', '2026-08-13', 1, 0)",
        [],
    )
    .unwrap();

    // Re-running schema initialization is the supported old-backup upgrade path.
    my_todos_lib::db::schema::initialize_schema(&conn).unwrap();
    let event = calendar_service::list_events(&conn, "2026-08-13", "2026-08-13")
        .unwrap()
        .remove(0);
    assert_eq!(event.start_date, "2026-08-13");
    assert_eq!(event.end_date, "2026-08-14");
    assert!(event.is_all_day);
}

#[test]
fn creates_updates_and_deletes_timed_events() {
    let db = setup_test_db();
    let conn = db.lock();
    let mut event_input = input("Focus", "2026-08-13", "2026-08-14");
    event_input.is_all_day = false;
    event_input.start_time = Some("09:00".into());
    event_input.end_time = Some("10:00".into());
    event_input.start_at = Some(1_776_049_200);
    event_input.end_at = Some(1_776_052_800);
    let created = calendar_service::create_event(&conn, event_input.clone()).unwrap();
    assert_eq!(created.start_time.as_deref(), Some("09:00"));

    event_input.title = "Deep focus".into();
    event_input.end_time = Some("10:30".into());
    let updated = calendar_service::update_event(&conn, created.id, event_input).unwrap();
    assert_eq!(updated.title, "Deep focus");
    assert_eq!(updated.end_time.as_deref(), Some("10:30"));

    calendar_service::delete_event(&conn, created.id).unwrap();
    assert!(
        calendar_service::list_events(&conn, "2026-08-13", "2026-08-13")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn expands_selected_weekdays_and_honors_count() {
    let rule = RecurrenceRule {
        frequency: RecurrenceFrequency::Weekly,
        interval: 1,
        weekdays: vec![1, 3], // Monday and Wednesday, Sunday = 0.
        until: None,
        count: Some(4),
    };
    let dates = calendar_service::recurrence_dates(
        date("2026-08-10"),
        &rule,
        date("2026-08-01"),
        date("2026-09-01"),
    )
    .unwrap();
    assert_eq!(
        dates,
        vec![
            date("2026-08-10"),
            date("2026-08-12"),
            date("2026-08-17"),
            date("2026-08-19"),
        ]
    );
}

#[test]
fn clamps_monthly_recurrence_to_short_months() {
    let rule = RecurrenceRule {
        frequency: RecurrenceFrequency::Monthly,
        interval: 1,
        weekdays: Vec::new(),
        until: None,
        count: Some(3),
    };
    let dates = calendar_service::recurrence_dates(
        date("2026-01-31"),
        &rule,
        date("2026-01-01"),
        date("2026-04-01"),
    )
    .unwrap();
    assert_eq!(
        dates,
        vec![date("2026-01-31"), date("2026-02-28"), date("2026-03-31")]
    );
}

#[test]
fn expands_recurring_series_only_inside_requested_range() {
    let db = setup_test_db();
    let conn = db.lock();
    let mut recurring = input("Standup", "2026-08-10", "2026-08-11");
    recurring.recurrence_rule = Some(
        serde_json::to_string(&RecurrenceRule {
            frequency: RecurrenceFrequency::Daily,
            interval: 2,
            weekdays: Vec::new(),
            until: Some("2026-08-20".into()),
            count: None,
        })
        .unwrap(),
    );
    calendar_service::create_event(&conn, recurring).unwrap();
    let events = calendar_service::list_events(&conn, "2026-08-13", "2026-08-17").unwrap();
    assert_eq!(
        events
            .iter()
            .map(|event| event.start_date.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-08-14", "2026-08-16"]
    );
    assert!(events.iter().all(|event| event.series_id == event.id));
    assert!(events
        .iter()
        .all(|event| event.series_start_date == "2026-08-10"));
}

#[test]
fn stores_planned_duration_with_task_schedule() {
    let db = setup_test_db();
    let task_id = create_test_task(&db, None, None, "Plan me");
    let conn = db.lock();
    let task = my_todos_lib::services::tasks_service::set_task_schedule(
        &conn,
        task_id,
        Some("2026-08-13T09:00:00".into()),
        Some(45),
    )
    .unwrap();
    assert_eq!(task.planned_duration_minutes, Some(45));
    assert!(my_todos_lib::services::tasks_service::set_task_schedule(
        &conn,
        task_id,
        task.deadline,
        Some(2),
    )
    .is_err());
}
