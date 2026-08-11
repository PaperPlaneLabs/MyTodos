use super::system_events::{
    handle_away_ended, handle_away_started, handle_away_started_with_reason,
};
use crate::db::DbConnection;
use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_foundation::string::CFString;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

// IOKit constants
const KIO_MESSAGE_CAN_SYSTEM_SLEEP: u32 = 0xe0000270;
const KIO_MESSAGE_SYSTEM_WILL_SLEEP: u32 = 0xe0000280;
const KIO_MESSAGE_SYSTEM_HAS_POWERED_ON: u32 = 0xe0000300;

type IoConnect = u32;
type IoService = u32;
type IoObject = u32;
type Natural = u32;
type IoReturn = i32;
type PowerCallback = extern "C" fn(*mut c_void, IoService, Natural, *mut c_void);

// FFI declarations for IOKit and CFNotificationCenter
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IORegisterForSystemPower(
        refcon: *mut c_void,
        port_ref: *mut *mut c_void,
        callback: PowerCallback,
        notifier: *mut IoObject,
    ) -> IoConnect;

    fn IOAllowPowerChange(root_port: IoConnect, notification_id: isize) -> IoReturn;

    fn IONotificationPortGetRunLoopSource(port: *mut c_void) -> *mut c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFNotificationCenterGetDistributedCenter() -> *mut c_void;
    fn CFNotificationCenterAddObserver(
        center: *mut c_void,
        observer: *const c_void,
        callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void),
        name: *mut c_void,
        object: *mut c_void,
        suspension_behavior: i32,
    );
}

struct MacOSListenerContext {
    app_handle: Arc<Mutex<AppHandle>>,
    db: Arc<Mutex<DbConnection>>,
    root_port: IoConnect,
}

extern "C" fn power_callback(
    refcon: *mut c_void,
    _service: IoService,
    message_type: Natural,
    message_argument: *mut c_void,
) {
    if message_type == KIO_MESSAGE_SYSTEM_WILL_SLEEP {
        println!("macOS system sleep detected");

        unsafe {
            let context = &*(refcon as *const MacOSListenerContext);

            if let (Ok(app), Ok(db)) = (context.app_handle.lock(), context.db.lock()) {
                handle_away_started_with_reason(&app, &db, super::AutoPauseReason::SystemSleep);
            }
        }
    } else if message_type == KIO_MESSAGE_SYSTEM_HAS_POWERED_ON {
        println!("macOS system wake detected");

        unsafe {
            let context = &*(refcon as *const MacOSListenerContext);

            if let (Ok(app), Ok(db)) = (context.app_handle.lock(), context.db.lock()) {
                handle_away_ended(&app, &db);
            }
        }
    }

    if matches!(
        message_type,
        KIO_MESSAGE_CAN_SYSTEM_SLEEP | KIO_MESSAGE_SYSTEM_WILL_SLEEP
    ) {
        unsafe {
            let context = &*(refcon as *const MacOSListenerContext);
            let _ = IOAllowPowerChange(context.root_port, message_argument as isize);
        }
    }
}

extern "C" fn lock_callback(
    _center: *mut c_void,
    observer: *mut c_void,
    name: *mut c_void,
    _object: *mut c_void,
    _user_info: *mut c_void,
) {
    unsafe {
        let name = CFString::wrap_under_get_rule(name as *mut _);
        let name_str = name.to_string();
        println!("macOS notification received: {}", name_str);

        let context = &*(observer as *const MacOSListenerContext);

        if name_str == "com.apple.screenIsLocked" {
            if let (Ok(app), Ok(db)) = (context.app_handle.lock(), context.db.lock()) {
                handle_away_started(&app, &db);
            }
        } else if name_str == "com.apple.screenIsUnlocked" {
            if let (Ok(app), Ok(db)) = (context.app_handle.lock(), context.db.lock()) {
                handle_away_ended(&app, &db);
            }
        }
    }
}

/// Initialize macOS power and lock event listener
pub fn initialize_macos_listener(app_handle: AppHandle, db: DbConnection) {
    std::thread::spawn(move || {
        println!("Initializing macOS power and lock event listener");

        let context = Box::new(MacOSListenerContext {
            app_handle: Arc::new(Mutex::new(app_handle)),
            db: Arc::new(Mutex::new(db)),
            root_port: 0,
        });
        let context_ptr = Box::into_raw(context);

        unsafe {
            // Register for system power events (sleep)
            let mut port_ref: *mut c_void = std::ptr::null_mut();
            let mut notifier: IoObject = 0;

            let root_port = IORegisterForSystemPower(
                context_ptr as *mut c_void,
                &mut port_ref,
                power_callback,
                &mut notifier,
            );
            (*context_ptr).root_port = root_port;

            // Register for screen lock/unlock notifications
            let center = CFNotificationCenterGetDistributedCenter();
            if !center.is_null() {
                let lock_name = CFString::from_static_string("com.apple.screenIsLocked");
                let unlock_name = CFString::from_static_string("com.apple.screenIsUnlocked");

                CFNotificationCenterAddObserver(
                    center,
                    context_ptr as *const c_void,
                    lock_callback,
                    lock_name.as_concrete_TypeRef() as *mut _,
                    std::ptr::null_mut(),
                    0, // CFNotificationSuspensionBehaviorDeliverImmediately
                );

                CFNotificationCenterAddObserver(
                    center,
                    context_ptr as *const c_void,
                    lock_callback,
                    unlock_name.as_concrete_TypeRef() as *mut _,
                    std::ptr::null_mut(),
                    0,
                );
            }

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
                CFRunLoop::run_current();
            } else {
                eprintln!("Failed to register for macOS power events");
            }
        }
    });
}
