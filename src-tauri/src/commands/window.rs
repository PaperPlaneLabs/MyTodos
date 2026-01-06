use crate::db::{DbConnection, WindowState};
use crate::error::{Result, AppError};
use tauri::{State, WebviewWindow, LogicalPosition, LogicalSize};

fn get_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn minimize_window(window: WebviewWindow) -> Result<()> {
    window.minimize().map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn toggle_maximize(window: WebviewWindow) -> Result<()> {
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| AppError::Other(e.to_string()))
    } else {
        window.maximize().map_err(|e| AppError::Other(e.to_string()))
    }
}

#[tauri::command]
pub fn close_window(window: WebviewWindow) -> Result<()> {
    window.close().map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn dock_window(window: WebviewWindow, side: String) -> Result<()> {
    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let screen_size = monitor.size();
    let scale_factor = monitor.scale_factor();
    
    // Logical size for the window
    let logical_width = 380.0;
    let logical_height = (screen_size.height as f64) / scale_factor;
    
    let screen_logical_width = (screen_size.width as f64) / scale_factor;

    let pos = if side == "left" {
        LogicalPosition::new(0.0, 0.0)
    } else {
        LogicalPosition::new(screen_logical_width - logical_width, 0.0)
    };

    window.set_size(LogicalSize::new(logical_width, logical_height))
        .map_err(|e| AppError::Other(e.to_string()))?;
    
    window.set_position(pos)
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn start_window_drag(window: WebviewWindow) -> Result<()> {
    window.start_dragging().map_err(|e| AppError::Other(e.to_string()))
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
