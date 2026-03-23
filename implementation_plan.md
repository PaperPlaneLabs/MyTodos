## Timer Recovery Plan

### Why
- A timer can survive an OS shutdown if the shutdown event is missed or the app is terminated before the pause write completes.
- On the next boot, the app treats that timer as still running and overcounts overnight time.

### What
- Add a backend-owned heartbeat for active running timers.
- Recover stale running timers during app startup before the heartbeat loop begins.
- Cap recovered time at the last known heartbeat so we do not count offline hours.

### Components
- `src-tauri/src/db/schema.rs`
  - Persist a `last_heartbeat_at` timestamp for the active timer.
- `src-tauri/src/commands/timer.rs`
  - Set and maintain heartbeat timestamps.
  - Add reusable stale-timer recovery helpers.
- `src-tauri/src/lib.rs`
  - Run stale-timer recovery during startup.
  - Start a lightweight background heartbeat loop.
- `src-tauri/tests/timer_tests.rs`
  - Verify stale running timers are auto-paused and capped to the last heartbeat.
