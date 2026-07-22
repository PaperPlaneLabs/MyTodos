use crate::db::{initialize_schema, DbConnection};
use crate::{events, services};
use tauri::{App, AppHandle, Emitter, Manager, Wry};

pub fn apply_saved_window_dock_preference(app: &App<Wry>, db: &DbConnection) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    match crate::commands::get_saved_window_dock_preference(db) {
        Ok(Some(dock_preference)) => {
            if let Err(error) =
                crate::commands::apply_dock_preference_to_window(&window, &dock_preference)
            {
                eprintln!(
                    "Failed to apply saved window dock preference '{}': {}",
                    dock_preference, error
                );
            }
        }
        Ok(None) => {}
        Err(error) => eprintln!("Failed to load saved window dock preference: {}", error),
    }
}

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
    events::initialize_system_listeners(app_handle.clone(), db.clone());
    services::window_tracking_service::initialize_tracker(db.clone());
    services::backup_service::initialize_backup(db.clone());

    match services::timer_service::recover_stale_active_timer(&db) {
        Ok(true) => println!("Recovered stale active timer during startup."),
        Ok(false) => {}
        Err(error) => eprintln!("Failed to recover stale active timer: {}", error),
    }

    let heartbeat_db = db.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(
            services::timer_service::ACTIVE_TIMER_HEARTBEAT_INTERVAL_SECONDS,
        ));

        if let Err(error) = services::timer_service::heartbeat_active_timer(&heartbeat_db) {
            eprintln!("Failed to update active timer heartbeat: {}", error);
        }
    });

    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            services::timer_service::TIMED_TIMER_CHECK_INTERVAL_SECONDS,
        ));

        loop {
            interval.tick().await;
            let now = chrono::Utc::now().timestamp();
            match services::timer_service::finish_expired_timed_timer_at(&db, now) {
                Ok(Some(completion)) => {
                    if let Err(error) = app_handle.emit("timer:finished", &completion) {
                        eprintln!("Failed to emit task timer completion: {error}");
                    }

                    if let Err(error) = crate::commands::window::open_task_timer_finished_window(
                        app_handle.clone(),
                        completion.task_id,
                        completion.task_title,
                        completion.duration_seconds,
                        completion.finished_at,
                    )
                    .await
                    {
                        eprintln!("Failed to open task timer completion window: {error}");
                    }
                }
                Ok(None) => {}
                Err(error) => eprintln!("Failed to finalize expired task timer: {error}"),
            }
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
