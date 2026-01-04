use crate::db::{DbConnection, WindowState};
use crate::error::Result;
use tauri::State;

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn save_window_state(
    db: State<DbConnection>,
    x: Option<i32>,
    y: Option<i32>,
    width: i32,
    height: i32,
) -> Result<()> {
    let conn = db.lock();
    let now = get_timestamp();

    conn.execute(
        "INSERT OR REPLACE INTO window_state (id, x, y, width, height, updated_at)
         VALUES (1, ?, ?, ?, ?, ?)",
        (x, y, width, height, now),
    )?;

    Ok(())
}

#[tauri::command]
pub fn get_window_state(db: State<DbConnection>) -> Result<Option<WindowState>> {
    let conn = db.lock();

    let result = conn.query_row(
        "SELECT x, y, width, height, updated_at FROM window_state WHERE id = 1",
        [],
        |row| {
            Ok(WindowState {
                x: row.get(0)?,
                y: row.get(1)?,
                width: row.get(2)?,
                height: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(state) => Ok(Some(state)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
