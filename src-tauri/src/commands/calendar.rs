use crate::db::{
    models::{CalendarEvent, Task},
    DbConnection,
};
use crate::error::Result;
use crate::google::GoogleCalendarState;
use rusqlite::{params, Connection, Row};
use tauri::State;

fn map_task_row(row: &Row<'_>) -> rusqlite::Result<Task> {
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
}

fn collect_tasks_by_deadline_range(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT t.* FROM tasks t
         WHERE t.deadline IS NOT NULL
         AND date(t.deadline) BETWEEN date(?1) AND date(?2)
         ORDER BY t.deadline, t.position",
    )?;

    let tasks = stmt.query_map([start_date, end_date], map_task_row)?;
    let tasks: std::result::Result<Vec<Task>, rusqlite::Error> = tasks.collect();
    Ok(tasks?)
}

fn collect_upcoming_deadline_tasks(conn: &Connection, start_date: &str) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT t.* FROM tasks t
         WHERE t.deadline IS NOT NULL
         AND t.completed = 0
         AND date(t.deadline) >= date(?1)
         ORDER BY t.deadline, t.position",
    )?;

    let tasks = stmt.query_map([start_date], map_task_row)?;
    let tasks: std::result::Result<Vec<Task>, rusqlite::Error> = tasks.collect();
    Ok(tasks?)
}

#[tauri::command]
pub fn get_tasks_by_deadline_range(
    db: State<DbConnection>,
    start_date: String,
    end_date: String,
) -> Result<Vec<Task>> {
    let conn = db.lock();
    collect_tasks_by_deadline_range(&conn, &start_date, &end_date)
}

#[tauri::command]
pub fn get_upcoming_deadline_tasks(
    db: State<DbConnection>,
    start_date: String,
) -> Result<Vec<Task>> {
    let conn = db.lock();
    collect_upcoming_deadline_tasks(&conn, &start_date)
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
        if let Err(e) = crate::google::sync::sync_task_to_calendar(db, &google_state, task_id).await
        {
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

#[cfg(test)]
mod tests {
    use super::{collect_tasks_by_deadline_range, collect_upcoming_deadline_tasks};
    use rusqlite::{params, Connection};

    fn setup_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                project_id INTEGER,
                section_id INTEGER,
                title TEXT NOT NULL,
                description TEXT,
                completed INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                total_time_seconds INTEGER NOT NULL DEFAULT 0,
                deadline TEXT,
                google_event_id TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )
        .expect("create tasks table");
        conn
    }

    fn insert_task(
        conn: &Connection,
        id: i64,
        title: &str,
        completed: bool,
        position: i32,
        deadline: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO tasks (
                id, project_id, section_id, title, description, completed, position,
                total_time_seconds, deadline, google_event_id, created_at, updated_at
            ) VALUES (?1, NULL, NULL, ?2, NULL, ?3, ?4, 0, ?5, NULL, 0, 0)",
            params![id, title, completed as i64, position, deadline],
        )
        .expect("insert task");
    }

    #[test]
    fn deadline_range_includes_timed_deadlines_on_end_date() {
        let conn = setup_connection();
        insert_task(&conn, 1, "Start day", false, 0, Some("2026-04-22"));
        insert_task(
            &conn,
            2,
            "End day timed",
            false,
            0,
            Some("2026-04-28T18:30"),
        );
        insert_task(&conn, 3, "Outside range", false, 0, Some("2026-04-29"));

        let tasks =
            collect_tasks_by_deadline_range(&conn, "2026-04-22", "2026-04-28").expect("query");

        let titles: Vec<_> = tasks.into_iter().map(|task| task.title).collect();
        assert_eq!(titles, vec!["Start day", "End day timed"]);
    }

    #[test]
    fn upcoming_deadline_tasks_excludes_completed_and_past_items() {
        let conn = setup_connection();
        insert_task(&conn, 1, "Past", false, 0, Some("2026-04-21T10:00"));
        insert_task(&conn, 2, "Completed", true, 0, Some("2026-04-22"));
        insert_task(&conn, 3, "Today all day", false, 2, Some("2026-04-22"));
        insert_task(&conn, 4, "Today timed", false, 1, Some("2026-04-22T09:30"));
        insert_task(&conn, 5, "Tomorrow", false, 0, Some("2026-04-23"));
        insert_task(&conn, 6, "No deadline", false, 0, None);

        let tasks = collect_upcoming_deadline_tasks(&conn, "2026-04-22").expect("query");

        let titles: Vec<_> = tasks.into_iter().map(|task| task.title).collect();
        assert_eq!(titles, vec!["Today all day", "Today timed", "Tomorrow"]);
    }
}
