mod common;

use common::setup_test_db;
use my_todos_lib::db::schema::initialize_schema;
use my_todos_lib::services::{afk_categories_service, timer_service};

#[test]
fn new_database_seeds_default_afk_categories() {
    let db = setup_test_db();

    let categories = afk_categories_service::list_categories(&db).unwrap();

    assert_eq!(categories, vec!["Meeting", "Lunch", "Snack"]);
}

#[test]
fn migration_recovers_categories_from_existing_away_tasks() {
    let db = setup_test_db();
    {
        let conn = db.lock();
        conn.execute("DROP TABLE afk_categories", []).unwrap();
        conn.execute(
            "INSERT INTO projects
                (name, description, color, position, total_time_seconds, is_system, created_at, updated_at)
             VALUES ('Away', 'Automatically tracked away-from-keyboard time', '#f59e0b', 0, 0, 1, 1, 1)",
            [],
        )
        .unwrap();
        let project_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO tasks
                (project_id, title, completed, position, total_time_seconds, is_system, created_at, updated_at)
             VALUES (?, 'Others', 0, 0, 120, 1, 1, 1)",
            [project_id],
        )
        .unwrap();

        initialize_schema(&conn).unwrap();
    }

    let categories = afk_categories_service::list_categories(&db).unwrap();
    assert_eq!(categories, vec!["Meeting", "Lunch", "Snack", "Others"]);
}

#[test]
fn merging_legacy_categories_normalizes_and_deduplicates_names() {
    let db = setup_test_db();

    let categories = afk_categories_service::merge_categories(
        &db,
        vec!["  Deep   Work ".to_string(), "deep work".to_string()],
    )
    .unwrap();

    assert_eq!(
        categories
            .iter()
            .filter(|name| name.eq_ignore_ascii_case("Deep Work"))
            .count(),
        1
    );
}

#[test]
fn removing_category_keeps_historical_away_task_and_entries() {
    let db = setup_test_db();
    afk_categories_service::add_category(&db, "Exercise").unwrap();
    timer_service::log_afk_time(&db, "Exercise", 90).unwrap();

    afk_categories_service::remove_category(&db, "Exercise").unwrap();

    let categories = afk_categories_service::list_categories(&db).unwrap();
    assert!(!categories.iter().any(|name| name == "Exercise"));

    let conn = db.lock();
    let (task_count, entry_count): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(DISTINCT t.id), COUNT(te.id)
             FROM tasks t
             JOIN projects p ON p.id = t.project_id
             LEFT JOIN time_entries te ON te.task_id = t.id
             WHERE p.name = 'Away' AND t.title = 'Exercise'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((task_count, entry_count), (1, 1));
}

#[test]
fn logging_unknown_afk_category_registers_it_for_future_use() {
    let db = setup_test_db();

    timer_service::log_afk_time(&db, "Phone call", 30).unwrap();

    let categories = afk_categories_service::list_categories(&db).unwrap();
    assert!(categories.iter().any(|name| name == "Phone call"));
}
