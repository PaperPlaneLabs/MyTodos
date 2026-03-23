use crate::db::{initialize_schema, DbConnection};
use crate::{events, services};
use tauri::{App, AppHandle, Manager, Wry};

pub fn initialize_database_state(db_conn: &DbConnection) -> Result<(), String> {
    let conn = db_conn.lock();
    initialize_schema(&conn).map_err(|error| error.to_string())?;

    let thirty_days_ago = chrono::Utc::now().timestamp() - (30 * 24 * 60 * 60);
    if let Err(error) = conn.execute(
        "DELETE FROM tasks WHERE completed = 1 AND updated_at < ?",
        [thirty_days_ago],
    ) {
        eprintln!("Failed to cleanup old tasks: {}", error);
    }

    Ok(())
}

pub fn initialize_runtime(app_handle: AppHandle, db: DbConnection) {
    events::initialize_system_listeners(app_handle, db.clone());

    match services::timer_service::recover_stale_active_timer(&db) {
        Ok(true) => println!("Recovered stale active timer during startup."),
        Ok(false) => {}
        Err(error) => eprintln!("Failed to recover stale active timer: {}", error),
    }

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(
            services::timer_service::ACTIVE_TIMER_HEARTBEAT_INTERVAL_SECONDS,
        ));

        if let Err(error) = services::timer_service::heartbeat_active_timer(&db) {
            eprintln!("Failed to update active timer heartbeat: {}", error);
        }
    });
}

pub fn apply_launch_visibility(app: &App<Wry>) {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--hidden".to_string()) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
}
