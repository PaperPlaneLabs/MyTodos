# Task Timer Walkthrough

## Implemented

- Added `Set Task Timer...` to active task context menus.
- Added 15/25/30/45/60-minute presets and a 1–1440 minute custom duration.
- Extended the existing singleton timer with nullable limit, remaining, and
  expiry state; unrestricted timers remain unchanged.
- Added pause/resume-safe countdown behavior and exact, idempotent expiry
  finalization through normal timer time entries.
- Added a Rust-side one-second expiry supervisor, `timer:finished` event, and an
  async always-on-top SPA completion window.
- Added `Switch task` and `Continue without timer` actions. Switching focuses
  the main window and returns it to the task surface.
- Added countdown/progress UI to the active timer widget.
- Suppressed and closed general break reminders for bounded sessions.
- Capped active statistics and daily display totals at the timer deadline.
- Preserved heartbeat-based stale recovery so offline time is not counted.

## Verification

- `npm run check`: 0 errors and 0 warnings.
- `npm run test -- --run`: 55 tests passed, including custom one-minute input
  and completion-window action ordering.
- `npm run build`: production frontend build passed.
- `cargo fmt -- --check`: passed.
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: full Rust suite passed, including 7 bounded-timer integration
  tests and the active-statistics expiry-cap unit test.
- `graphify update .`: rebuilt the project graph with 1309 nodes and 2844 edges.

## Notes

- Fixed inert completion-window actions by granting the
  `task-timer-finished` window the same core desktop capabilities as the other
  SPA windows. `Switch task` now restores the main window before focusing it,
  while `Continue without timer` starts the task through the ordinary timer
  command and emits a refresh event to the main UI.
- Fixed numeric custom-duration input handling: Svelte number inputs produce
  numbers, so the setup modal no longer calls string methods on the bound value.
- The full Clippy gate exposed two pre-existing `unnecessary_sort_by` warnings
  in window tracking. They were mechanically changed to equivalent
  `sort_by_key(Reverse(...))` calls so the required lint gate could pass.
- No live GUI smoke test was run because the desktop app uses the user's real
  application database path. Backend lifecycle behavior, frontend wiring, SPA
  compilation, and production bundling are covered by the checks above.

---

## Durable AFK Categories

- Added an `afk_categories` SQLite table as the shared source of truth for
  development and installed desktop builds.
- On upgrade, seeded the table from Meeting/Lunch/Snack plus every existing task
  under the Away project, preserving categories such as Others and miss.
- Migrated each WebView origin's legacy `afkCategories` localStorage value into
  SQLite, then removed the legacy key only after a successful merge.
- Added list, merge, add, and remove Tauri commands through the standard `db.ts`
  bridge and converted Settings/Resume category loading to asynchronous SQLite
  reads.
- Kept removal non-destructive: it hides the picker option without deleting its
  Away task or historical time entries.
- AFK logging now registers unknown category names automatically, and restoring
  an older backup reruns schema migrations before refreshing the category store.

### AFK Verification

- `npm run check`: 0 errors and 0 warnings.
- `npm run test -- --run`: 57 frontend tests passed.
- `npm run build`: production frontend build passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 120 Rust tests passed, including 5 new AFK persistence tests and
  an older-backup migration regression.
- `graphify update .`: rebuilt the project graph with 1350 nodes and 2934 edges.
