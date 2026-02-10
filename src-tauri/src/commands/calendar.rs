use crate::db::{
    models::{CalendarEvent, Task},
    DbConnection,
};
use crate::error::Result;
use crate::google::GoogleCalendarState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_tasks_by_deadline_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<Task>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT t.* FROM tasks t
         WHERE t.deadline BETWEEN ?1 AND ?2
         AND t.deadline IS NOT NULL
         ORDER BY t.deadline, t.position",
    )?;

    let tasks = stmt.query_map([start_date, end_date], |row| {
        Ok(Task {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            section_id: row.get("section_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            completed: row.get("completed")?,
            position: row.get("position")?,
            total_time_seconds: row.get("total_time_seconds")?,
            deadline: row.get("deadline")?,
            google_event_id: row.get("google_event_id")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    })?;

    let tasks: std::result::Result<Vec<Task>, rusqlite::Error> = tasks.collect();
    Ok(tasks?)
}

#[tauri::command]
pub fn update_task_deadline(
    db: State<DbConnection>,
    google_state: State<GoogleCalendarState>,
    task_id: i64,
    deadline: Option<String>,
) -> Result<()> {
    let conn = db.lock();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE tasks SET deadline = ?1, updated_at = ?2 WHERE id = ?3",
        params![deadline, now, task_id],
    )?;
    drop(conn);

    // Fire-and-forget: sync to Google Calendar
    let db = db.inner().clone();
    let google_state = google_state.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::google::sync::sync_task_to_calendar(db, &google_state, task_id).await {
            eprintln!("Failed to sync deadline update to Google Calendar: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn create_calendar_event(
    db: State<DbConnection>,
    title: String,
    description: Option<String>,
    date: String,
    is_all_day: bool,
    color: Option<String>,
) -> Result<CalendarEvent> {
    let conn = db.lock();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO calendar_events (title, description, date, is_all_day, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![title, description, date, is_all_day as i64, color, now],
    )?;

    let id = conn.last_insert_rowid();
    Ok(CalendarEvent {
        id,
        title,
        description,
        date,
        is_all_day,
        color: color.unwrap_or_default(),
    })
}

#[tauri::command]
pub fn get_calendar_events_in_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<CalendarEvent>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT * FROM calendar_events
         WHERE date BETWEEN ?1 AND ?2
         ORDER BY date, is_all_day DESC",
    )?;

    let events = stmt.query_map([start_date, end_date], |row| {
        Ok(CalendarEvent {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            date: row.get(3)?,
            is_all_day: row.get::<_, i64>(4)? == 1,
            color: row.get(5)?,
        })
    })?;

    let events: std::result::Result<Vec<CalendarEvent>, rusqlite::Error> = events.collect();
    Ok(events?)
}
