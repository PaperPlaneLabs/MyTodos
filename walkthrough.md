## Timer Recovery Walkthrough

### What changed
- Added `last_heartbeat_at` to the `active_timer` record so the app can tell how recently a running timer was confirmed alive.
- Added backend helpers to:
  - update the heartbeat while a timer is running,
  - detect stale running timers,
  - auto-pause stale timers at the last heartbeat instead of the current time.
- Startup now runs stale-timer recovery before the background heartbeat loop begins.

### Why this fixes the overnight issue
- If Windows shutdown or process termination prevents the normal auto-pause handler from finishing, the timer row may still be marked as running.
- On the next app launch, the stale-timer recovery sees that the heartbeat is old and pauses the timer using the last confirmed heartbeat.
- That means offline hours are no longer added to the task.

### Verification
- `cargo test --test timer_tests`
  - Passed all 24 timer tests.
  - Includes coverage for:
    - stale timers being recovered and capped to `last_heartbeat_at`,
    - recent running timers remaining active.
- `npm run check`
  - Completed with 0 errors.
  - Reported 6 existing accessibility warnings in `AppHeader.svelte` and `SettingsView.svelte`.

### Notes
- I kept the existing OS event-based auto-pause behavior in place.
- The new heartbeat recovery is a fallback for missed shutdown/crash cases, not a replacement for lock/sleep/shutdown listeners.
