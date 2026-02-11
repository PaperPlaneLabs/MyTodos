use super::common::get_timestamp;
use crate::db::{DbConnection, WindowState};
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::{LogicalPosition, LogicalSize, State, WebviewWindow};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowOrientation {
    pub side: String,
    pub is_portrait: bool,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
pub fn minimize_window(window: WebviewWindow) -> Result<()> {
    window
        .minimize()
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn toggle_maximize(window: WebviewWindow) -> Result<()> {
    if window.is_maximized().unwrap_or(false) {
        window
            .unmaximize()
            .map_err(|e| AppError::Other(e.to_string()))
    } else {
        window
            .maximize()
            .map_err(|e| AppError::Other(e.to_string()))
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

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();

    // Convert physical work area to logical
    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_y = (work_area.position.y as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let logical_work_height = (work_area.size.height as f64) / scale_factor;

    // Logical size for the window
    let logical_width = 380.0;
    let logical_height = logical_work_height;

    let pos = if side == "left" {
        LogicalPosition::new(logical_work_x, logical_work_y)
    } else {
        LogicalPosition::new(
            logical_work_x + logical_work_width - logical_width,
            logical_work_y,
        )
    };

    window
        .set_size(LogicalSize::new(logical_width, logical_height))
        .map_err(|e| AppError::Other(e.to_string()))?;

    window
        .set_position(pos)
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn set_collapsed(window: WebviewWindow, collapsed: bool, top: f64) -> Result<()> {
    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();

    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_y = (work_area.position.y as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let logical_work_height = (work_area.size.height as f64) / scale_factor;

    let full_width = 380.0;
    let handle_width = 44.0;
    let handle_height = 140.0;

    if collapsed {
        window
            .set_min_size(Some(LogicalSize::new(handle_width, handle_height)))
            .map_err(|e| AppError::Other(e.to_string()))?;

        let x = logical_work_x + logical_work_width - handle_width;

        window
            .set_size(LogicalSize::new(handle_width, handle_height))
            .map_err(|e| AppError::Other(e.to_string()))?;
        window
            .set_position(LogicalPosition::new(x, logical_work_y + top))
            .map_err(|e| AppError::Other(e.to_string()))?;
        window
            .set_always_on_top(true)
            .map_err(|e| AppError::Other(e.to_string()))?;
    } else {
        window
            .set_min_size(Some(LogicalSize::new(320.0, 400.0)))
            .map_err(|e| AppError::Other(e.to_string()))?;

        let x = logical_work_x + logical_work_width - full_width;

        window
            .set_size(LogicalSize::new(full_width, logical_work_height))
            .map_err(|e| AppError::Other(e.to_string()))?;
        window
            .set_position(LogicalPosition::new(x, logical_work_y))
            .map_err(|e| AppError::Other(e.to_string()))?;
        window
            .set_always_on_top(false)
            .map_err(|e| AppError::Other(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub fn move_window(window: WebviewWindow, x: f64, y: f64) -> Result<()> {
    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| AppError::Other(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn start_window_drag(window: WebviewWindow) -> Result<()> {
    window
        .start_dragging()
        .map_err(|e| AppError::Other(e.to_string()))
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

#[tauri::command]
pub fn get_window_orientation(window: WebviewWindow) -> Result<WindowOrientation> {
    let monitor = window
        .current_monitor()
        .map_err(|e| AppError::Other(e.to_string()))?
        .ok_or_else(|| AppError::Other("Could not find current monitor".to_string()))?;

    let scale_factor = monitor.scale_factor();
    let work_area = monitor.work_area();
    let window_position = window
        .outer_position()
        .map_err(|e| AppError::Other(e.to_string()))?;
    let window_size = window
        .inner_size()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let logical_work_x = (work_area.position.x as f64) / scale_factor;
    let logical_work_width = (work_area.size.width as f64) / scale_factor;
    let window_x = (window_position.x as f64) / scale_factor;
    let window_width = (window_size.width as f64) / scale_factor;

    let threshold = 50.0;
    let side = if window_x <= logical_work_x + threshold {
        "left"
    } else if window_x + window_width >= logical_work_x + logical_work_width - threshold {
        "right"
    } else {
        "center"
    };

    let is_portrait = window_size.height > window_size.width;

    Ok(WindowOrientation {
        side: side.to_string(),
        is_portrait,
        width: window_width,
        height: (window_size.height as f64) / scale_factor,
    })
}
