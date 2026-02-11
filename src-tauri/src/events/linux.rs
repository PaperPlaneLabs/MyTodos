use super::{auto_pause_if_running, AutoPauseReason};
use crate::db::DbConnection;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Mutex;
use zbus::{Connection, MatchRule};

/// Initialize Linux systemd D-Bus listener
pub fn initialize_linux_listener(app_handle: AppHandle, db: DbConnection) {
    tokio::spawn(async move {
        println!("Initializing Linux systemd D-Bus listener");

        let app_handle = Arc::new(Mutex::new(app_handle));
        let db = Arc::new(Mutex::new(db));

        match Connection::system().await {
            Ok(connection) => {
                // Subscribe to PrepareForSleep signal from systemd
                let rule = MatchRule::builder()
                    .msg_type(zbus::message::Type::Signal)
                    .interface("org.freedesktop.login1.Manager")
                    .unwrap()
                    .member("PrepareForSleep")
                    .unwrap()
                    .build();

                match connection.add_match(rule).await {
                    Ok(mut stream) => {
                        println!("Successfully subscribed to systemd PrepareForSleep signal");

                        while let Some(msg) = stream.next().await {
                            // PrepareForSleep signal has a boolean argument:
                            // true = about to sleep, false = waking up
                            if let Ok(body) = msg.body() {
                                if let Ok(is_sleeping) = body.deserialize::<bool>() {
                                    if is_sleeping {
                                        println!("Linux system suspend detected");

                                        let app = app_handle.lock().await;
                                        let db = db.lock().await;
                                        auto_pause_if_running(
                                            &app,
                                            &db,
                                            AutoPauseReason::SystemSleep,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to subscribe to PrepareForSleep signal: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to system D-Bus: {}", e);
            }
        }
    });
}
