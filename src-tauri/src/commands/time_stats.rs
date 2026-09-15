use crate::db::DbConnection;
use crate::error::Result;
use crate::services::time_service;
pub use crate::services::time_service::{DailyAggregate, ProjectTime, TaskTimeEntry, TimeStats};
use tauri::State;

#[tauri::command]
pub fn get_time_stats(db: State<DbConnection>, include_active_timer: bool) -> Result<TimeStats> {
    let conn = db.lock();
    time_service::get_time_stats(&conn, include_active_timer)
}

#[cfg(test)]
mod tests {
    use crate::db::initialize_schema;
    use crate::services::time_service::get_active_timer_duration;
    use rusqlite::Connection;

    #[test]
    fn active_timed_timer_stats_are_capped_at_expiry() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        initialize_schema(&conn).unwrap();
        let now = chrono::Local::now().timestamp();

        conn.execute(
            "INSERT INTO projects (id, name, color, position, created_at, updated_at)
             VALUES (1, 'Focus', '#6366f1', 0, ?, ?)",
            (now, now),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (
                 id, project_id, title, completed, position, created_at, updated_at
             ) VALUES (1, 1, 'Deep work', 0, 0, ?, ?)",
            (now, now),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO active_timer (
                 id, task_id, started_at, elapsed_seconds, is_running,
                 last_heartbeat_at, project_id, timer_limit_seconds,
                 timer_remaining_seconds, timer_expires_at
             ) VALUES (1, 1, ?, 0, 1, ?, 1, 60, 60, ?)",
            (now - 120, now, now - 60),
        )
        .unwrap();

        let row = get_active_timer_duration(&conn).unwrap().unwrap();
        assert_eq!(row.5, 60);
    }
}
