mod common;

use common::*;
use my_todos_lib::db::TimeEntry;
use my_todos_lib::error::{AppError, Result};

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

// Implementation of time_entries commands for testing
fn create_manual_entry_impl(
    db: &DbConnection,
    task_id: i64,
    duration_seconds: i64,
    note: Option<String>,
) -> Result<TimeEntry> {
    let conn = db.lock();

    let task_exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE id = ?", [task_id], |_| Ok(true))
        .unwrap_or(false);

    if !task_exists {
        return Err(AppError::NotFound(format!(
            "Task with id {} not found",
            task_id
        )));
    }

    if duration_seconds <= 0 {
        return Err(AppError::InvalidInput(
            "Duration must be positive".to_string(),
        ));
    }

    let now = get_timestamp();

    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, note, created_at)
         VALUES (?, 'manual', ?, ?, ?)",
        (task_id, duration_seconds, &note, now),
    )?;

    let id = conn.last_insert_rowid();

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration_seconds, task_id),
    )?;

    let project_id: i64 = conn.query_row(
        "SELECT project_id FROM tasks WHERE id = ?",
        [task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (duration_seconds, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row(
            "SELECT section_id FROM tasks WHERE id = ?",
            [task_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (duration_seconds, sid),
        )?;
    }

    Ok(TimeEntry {
        id,
        task_id,
        entry_type: "manual".to_string(),
        duration_seconds,
        started_at: None,
        ended_at: None,
        note,
        created_at: now,
    })
}

fn update_time_entry_impl(
    db: &DbConnection,
    id: i64,
    duration_seconds: i64,
    note: Option<String>,
) -> Result<()> {
    let conn = db.lock();

    if duration_seconds <= 0 {
        return Err(AppError::InvalidInput(
            "Duration must be positive".to_string(),
        ));
    }

    let old_duration: i64 = conn
        .query_row(
            "SELECT duration_seconds FROM time_entries WHERE id = ?",
            [id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::NotFound(format!("Time entry with id {} not found", id)))?;

    let task_id: i64 = conn.query_row(
        "SELECT task_id FROM time_entries WHERE id = ?",
        [id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE time_entries SET duration_seconds = ?, note = ? WHERE id = ?",
        (duration_seconds, note, id),
    )?;

    let diff = duration_seconds - old_duration;

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (diff, task_id),
    )?;

    let project_id: i64 = conn.query_row(
        "SELECT project_id FROM tasks WHERE id = ?",
        [task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (diff, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row(
            "SELECT section_id FROM tasks WHERE id = ?",
            [task_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
            (diff, sid),
        )?;
    }

    Ok(())
}

fn delete_time_entry_impl(db: &DbConnection, id: i64) -> Result<()> {
    let conn = db.lock();

    let (task_id, duration): (i64, i64) = conn
        .query_row(
            "SELECT task_id, duration_seconds FROM time_entries WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Time entry with id {} not found", id)))?;

    conn.execute("DELETE FROM time_entries WHERE id = ?", [id])?;

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
        (duration, task_id),
    )?;

    let project_id: i64 = conn.query_row(
        "SELECT project_id FROM tasks WHERE id = ?",
        [task_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
        (duration, project_id),
    )?;

    let section_id: Option<i64> = conn
        .query_row(
            "SELECT section_id FROM tasks WHERE id = ?",
            [task_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
            (duration, sid),
        )?;
    }

    Ok(())
}

fn reset_task_time_impl(db: &DbConnection, id: i64) -> Result<()> {
    let conn = db.lock();

    let (project_id, section_id, current_time): (i64, Option<i64>, i64) = conn
        .query_row(
            "SELECT project_id, section_id, total_time_seconds FROM tasks WHERE id = ?",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| AppError::NotFound(format!("Task with id {} not found", id)))?;

    conn.execute("DELETE FROM time_entries WHERE task_id = ?", [id])?;

    conn.execute("UPDATE tasks SET total_time_seconds = 0 WHERE id = ?", [id])?;

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
        (current_time, project_id),
    )?;

    if let Some(sid) = section_id {
        conn.execute(
            "UPDATE sections SET total_time_seconds = total_time_seconds - ? WHERE id = ?",
            (current_time, sid),
        )?;
    }

    Ok(())
}

// Tests for manual time entry creation
#[test]
fn test_create_manual_entry_success() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = create_manual_entry_impl(&db, task_id, 3600, Some("Test note".to_string()));

    assert!(result.is_ok());
    let entry = result.unwrap();
    assert_eq!(entry.task_id, task_id);
    assert_eq!(entry.duration_seconds, 3600);
    assert_eq!(entry.entry_type, "manual");
    assert_eq!(entry.note, Some("Test note".to_string()));

    // Verify cascading updates
    assert_eq!(get_task_time(&db, task_id), 3600);
    assert_eq!(get_project_time(&db, project_id), 3600);
}

#[test]
fn test_create_manual_entry_with_section() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task_id = create_test_task(&db, Some(project_id), Some(section_id), "Test Task");

    let result = create_manual_entry_impl(&db, task_id, 1800, None);

    assert!(result.is_ok());

    // Verify all three levels updated
    assert_eq!(get_task_time(&db, task_id), 1800);
    assert_eq!(get_section_time(&db, section_id), 1800);
    assert_eq!(get_project_time(&db, project_id), 1800);
}

#[test]
fn test_create_manual_entry_invalid_task() {
    let db = setup_test_db();

    let result = create_manual_entry_impl(&db, 999, 3600, None);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_create_manual_entry_zero_duration() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = create_manual_entry_impl(&db, task_id, 0, None);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::InvalidInput(_) => {}
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_create_manual_entry_negative_duration() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = create_manual_entry_impl(&db, task_id, -100, None);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::InvalidInput(_) => {}
        _ => panic!("Expected InvalidInput error"),
    }
}

// Tests for updating time entries
#[test]
fn test_update_time_entry_increase_duration() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let entry = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();

    let result = update_time_entry_impl(&db, entry.id, 5400, Some("Updated".to_string()));

    assert!(result.is_ok());

    // Task time should increase by 1800 (5400 - 3600)
    assert_eq!(get_task_time(&db, task_id), 5400);
    assert_eq!(get_project_time(&db, project_id), 5400);
}

#[test]
fn test_update_time_entry_decrease_duration() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let entry = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();

    let result = update_time_entry_impl(&db, entry.id, 1800, None);

    assert!(result.is_ok());

    // Task time should decrease by 1800 (1800 - 3600)
    assert_eq!(get_task_time(&db, task_id), 1800);
    assert_eq!(get_project_time(&db, project_id), 1800);
}

#[test]
fn test_update_time_entry_with_section() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task_id = create_test_task(&db, Some(project_id), Some(section_id), "Test Task");

    let entry = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();
    update_time_entry_impl(&db, entry.id, 7200, None).unwrap();

    // All three levels should reflect the new duration
    assert_eq!(get_task_time(&db, task_id), 7200);
    assert_eq!(get_section_time(&db, section_id), 7200);
    assert_eq!(get_project_time(&db, project_id), 7200);
}

#[test]
fn test_update_time_entry_invalid_id() {
    let db = setup_test_db();

    let result = update_time_entry_impl(&db, 999, 3600, None);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_update_time_entry_invalid_duration() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let entry = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();

    let result = update_time_entry_impl(&db, entry.id, -100, None);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::InvalidInput(_) => {}
        _ => panic!("Expected InvalidInput error"),
    }
}

// Tests for deleting time entries
#[test]
fn test_delete_time_entry_success() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let entry = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();

    let result = delete_time_entry_impl(&db, entry.id);

    assert!(result.is_ok());

    // Times should be back to zero
    assert_eq!(get_task_time(&db, task_id), 0);
    assert_eq!(get_project_time(&db, project_id), 0);
    assert_eq!(count_time_entries(&db, task_id), 0);
}

#[test]
fn test_delete_time_entry_with_section() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task_id = create_test_task(&db, Some(project_id), Some(section_id), "Test Task");

    let entry = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();
    delete_time_entry_impl(&db, entry.id).unwrap();

    // All three levels should be back to zero
    assert_eq!(get_task_time(&db, task_id), 0);
    assert_eq!(get_section_time(&db, section_id), 0);
    assert_eq!(get_project_time(&db, project_id), 0);
}

#[test]
fn test_delete_time_entry_invalid_id() {
    let db = setup_test_db();

    let result = delete_time_entry_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_delete_one_of_multiple_entries() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let entry1 = create_manual_entry_impl(&db, task_id, 3600, None).unwrap();
    let _entry2 = create_manual_entry_impl(&db, task_id, 1800, None).unwrap();

    delete_time_entry_impl(&db, entry1.id).unwrap();

    // Should only have entry2's time remaining
    assert_eq!(get_task_time(&db, task_id), 1800);
    assert_eq!(get_project_time(&db, project_id), 1800);
    assert_eq!(count_time_entries(&db, task_id), 1);
}

// Tests for reset_task_time
#[test]
fn test_reset_task_time_success() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Add multiple entries
    create_manual_entry_impl(&db, task_id, 3600, None).unwrap();
    create_manual_entry_impl(&db, task_id, 1800, None).unwrap();

    assert_eq!(get_task_time(&db, task_id), 5400);
    assert_eq!(get_project_time(&db, project_id), 5400);

    let result = reset_task_time_impl(&db, task_id);

    assert!(result.is_ok());

    // Everything should be reset
    assert_eq!(get_task_time(&db, task_id), 0);
    assert_eq!(get_project_time(&db, project_id), 0);
    assert_eq!(count_time_entries(&db, task_id), 0);
}

#[test]
fn test_reset_task_time_with_section() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let section_id = create_test_section(&db, project_id, "Test Section");
    let task_id = create_test_task(&db, Some(project_id), Some(section_id), "Test Task");

    create_manual_entry_impl(&db, task_id, 7200, None).unwrap();

    reset_task_time_impl(&db, task_id).unwrap();

    // All three levels reset
    assert_eq!(get_task_time(&db, task_id), 0);
    assert_eq!(get_section_time(&db, section_id), 0);
    assert_eq!(get_project_time(&db, project_id), 0);
}

#[test]
fn test_reset_task_time_invalid_task() {
    let db = setup_test_db();

    let result = reset_task_time_impl(&db, 999);

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::NotFound(msg) => assert!(msg.contains("999")),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_reset_task_time_already_zero() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    let result = reset_task_time_impl(&db, task_id);

    assert!(result.is_ok());
    assert_eq!(get_task_time(&db, task_id), 0);
    assert_eq!(get_project_time(&db, project_id), 0);
}

// Tests for complex cascading scenarios
#[test]
fn test_multiple_tasks_in_same_project() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task1_id = create_test_task(&db, Some(project_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task 2");

    create_manual_entry_impl(&db, task1_id, 3600, None).unwrap();
    create_manual_entry_impl(&db, task2_id, 1800, None).unwrap();

    assert_eq!(get_task_time(&db, task1_id), 3600);
    assert_eq!(get_task_time(&db, task2_id), 1800);
    assert_eq!(get_project_time(&db, project_id), 5400); // Sum of both tasks
}

#[test]
fn test_multiple_tasks_in_different_sections() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let section1_id = create_test_section(&db, project_id, "Section 1");
    let section2_id = create_test_section(&db, project_id, "Section 2");
    let task1_id = create_test_task(&db, Some(project_id), Some(section1_id), "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), Some(section2_id), "Task 2");

    create_manual_entry_impl(&db, task1_id, 3600, None).unwrap();
    create_manual_entry_impl(&db, task2_id, 1800, None).unwrap();

    assert_eq!(get_section_time(&db, section1_id), 3600);
    assert_eq!(get_section_time(&db, section2_id), 1800);
    assert_eq!(get_project_time(&db, project_id), 5400); // Sum of all tasks
}

#[test]
fn test_reset_one_task_in_multi_task_project() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task1_id = create_test_task(&db, Some(project_id), None, "Task 1");
    let task2_id = create_test_task(&db, Some(project_id), None, "Task 2");

    create_manual_entry_impl(&db, task1_id, 3600, None).unwrap();
    create_manual_entry_impl(&db, task2_id, 1800, None).unwrap();

    reset_task_time_impl(&db, task1_id).unwrap();

    // Task 1 reset, Task 2 unchanged
    assert_eq!(get_task_time(&db, task1_id), 0);
    assert_eq!(get_task_time(&db, task2_id), 1800);
    // Project only has Task 2's time
    assert_eq!(get_project_time(&db, project_id), 1800);
}

#[test]
fn test_mixed_manual_and_timer_entries() {
    let db = setup_test_db();
    let project_id = create_test_project(&db, "Test Project");
    let task_id = create_test_task(&db, Some(project_id), None, "Test Task");

    // Create manual entry
    create_manual_entry_impl(&db, task_id, 3600, None).unwrap();

    // Simulate timer entry (using direct DB insert)
    let conn = db.lock();
    let now = get_timestamp();
    conn.execute(
        "INSERT INTO time_entries (task_id, entry_type, duration_seconds, started_at, ended_at, created_at)
         VALUES (?, 'timer', ?, ?, ?, ?)",
        (task_id, 1800, now - 1800, now, now),
    )
    .unwrap();

    conn.execute(
        "UPDATE tasks SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (1800, task_id),
    )
    .unwrap();

    conn.execute(
        "UPDATE projects SET total_time_seconds = total_time_seconds + ? WHERE id = ?",
        (1800, project_id),
    )
    .unwrap();
    drop(conn);

    // Both entries should contribute
    assert_eq!(get_task_time(&db, task_id), 5400);
    assert_eq!(count_time_entries(&db, task_id), 2);

    // Reset should clear both
    reset_task_time_impl(&db, task_id).unwrap();
    assert_eq!(get_task_time(&db, task_id), 0);
    assert_eq!(count_time_entries(&db, task_id), 0);
}
