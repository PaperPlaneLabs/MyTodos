use super::{auto_pause_if_running, AutoPauseReason};
use crate::db::DbConnection;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

// IOKit constants
const KIO_MESSAGE_SYSTEM_WILL_SLEEP: u32 = 0xe0000280;

// FFI declarations for IOKit
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IORegisterForSystemPower(
        refcon: *mut c_void,
        port_ref: *mut *mut c_void,
        callback: extern "C" fn(*mut c_void, u32, u32),
        notifier: *mut *mut c_void,
    ) -> *mut c_void;

    fn IONotificationPortGetRunLoopSource(port: *mut c_void) -> *mut c_void;
}

struct MacOSListenerContext {
    app_handle: Arc<Mutex<AppHandle>>,
    db: Arc<Mutex<DbConnection>>,
}

extern "C" fn power_callback(refcon: *mut c_void, _service: u32, message_type: u32) {
    if message_type == KIO_MESSAGE_SYSTEM_WILL_SLEEP {
        println!("macOS system sleep detected");

        unsafe {
            let context = &*(refcon as *const MacOSListenerContext);

            if let (Ok(app), Ok(db)) = (context.app_handle.lock(), context.db.lock()) {
                auto_pause_if_running(&app, &db, AutoPauseReason::SystemSleep);
            }
        }
    }
}

/// Initialize macOS power event listener
pub fn initialize_macos_listener(app_handle: AppHandle, db: DbConnection) {
    std::thread::spawn(move || {
        println!("Initializing macOS power event listener");

        let context = Box::new(MacOSListenerContext {
            app_handle: Arc::new(Mutex::new(app_handle)),
            db: Arc::new(Mutex::new(db)),
        });

        unsafe {
            let mut port_ref: *mut c_void = std::ptr::null_mut();
            let mut notifier: *mut c_void = std::ptr::null_mut();

            let _root_port = IORegisterForSystemPower(
                Box::into_raw(context) as *mut c_void,
                &mut port_ref,
                power_callback,
                &mut notifier,
            );

            if !port_ref.is_null() {
                let run_loop_source = IONotificationPortGetRunLoopSource(port_ref);
                let run_loop = CFRunLoop::get_current();

                // Add source to run loop
                core_foundation::runloop::CFRunLoopAddSource(
                    run_loop.as_concrete_TypeRef(),
                    run_loop_source as *mut _,
                    kCFRunLoopCommonModes,
                );

                // Run the loop
                run_loop.run();
            } else {
                eprintln!("Failed to register for macOS power events");
            }
        }
    });
}
