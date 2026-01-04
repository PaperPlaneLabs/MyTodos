mod commands;
mod db;
mod error;

use db::{initialize_connection, initialize_schema, DbConnection};

#[tauri::command]
fn initialize_database(db: tauri::State<DbConnection>) -> Result<(), String> {
    let conn = db.lock();
    initialize_schema(&conn).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_conn = initialize_connection().expect("Failed to initialize database connection");

    {
        let conn = db_conn.lock();
        initialize_schema(&conn).expect("Failed to initialize database schema");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(db_conn)
        .invoke_handler(tauri::generate_handler![
            initialize_database,
            commands::get_all_projects,
            commands::get_project,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::reorder_projects,
            commands::get_project_stats,
            commands::get_sections_by_project,
            commands::create_section,
            commands::update_section,
            commands::delete_section,
            commands::reorder_sections,
            commands::get_tasks_by_project,
            commands::get_tasks_by_section,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            commands::toggle_task_completion,
            commands::reorder_tasks,
            commands::get_active_timer,
            commands::start_timer,
            commands::pause_timer,
            commands::resume_timer,
            commands::stop_timer,
            commands::reset_timer,
            commands::create_manual_entry,
            commands::get_time_entries_by_task,
            commands::update_time_entry,
            commands::delete_time_entry,
            commands::save_window_state,
            commands::get_window_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
