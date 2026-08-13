# Task Timer Walkthrough

## Calendar Redesign

- Rebuilt Calendar around a normalized item model for scheduled tasks, local
  events, read-only Google events, and an optional tracked-time overlay.
- Added a consolidated toolbar, persistent source filters, configurable Monday
  or Sunday week starts, a six-row Month view with readable item labels, and a
  responsive contextual inspector/editor.
- Replaced the Week agenda with a 24-hour scheduling grid featuring overlapping
  event layout, all-day items, the current-time marker, click-to-create,
  drag-to-reschedule, resize handles, and a focused portrait day layout.
- Added local all-day/timed event CRUD and daily, weekly, monthly, and yearly
  recurrence expansion. Recurring edits operate on the whole series in V1.
- Added Google range import with pagination, SQLite cache fallback, stale-state
  reporting, and deduplication of Todoz tasks already exported to Google.
- Preserved timed task scheduling with planned durations and made tracked-time
  sessions inspectable, editable, and deletable from Calendar.
- Migrated legacy local events without data loss and corrected calendar-range
  time-entry queries to use the session start timestamp.

### Calendar Verification

- `npm run check`: passed with 0 errors and 0 warnings.
- `npm run test -- --run`: 11 files and 82 tests passed, including four new
  date, week-start, normalization, and overlap-layout tests.
- `npm run build`: production SvelteKit build passed.
- `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: all Rust suites passed, including six new calendar migration,
  CRUD, recurrence, range, and planned-duration tests.
- `git diff --check`: passed; Git reported line-ending normalization notices only.
- `graphify update .`: rebuilt the project graph with 1,569 nodes and 3,451 edges.
- Desktop startup reached the Tauri development runtime successfully. A visual
  capture was unavailable because the saved window state kept the test window
  hidden, so UI verification is covered by compile, test, and build gates.

## Unified Today Agenda

- Replaced the separate Work Queue and Schedule cards with one Agenda card ordered as All day, Anytime, and Timeline.
- Added a discriminated agenda item model and pure deterministic sorting that places events before tasks at equal times.
- Reused Today task rows unchanged and added keyboard-accessible event buttons that select today and open Calendar without mutating the timer or event.
- Kept overdue work separate, progress task-only, the active timer above loading content, and the existing page-level scrolling behavior.

### Verification

- Focused Today Vitest run: 2 files and 9 tests passed.
- `npm run check`: 0 errors and 0 warnings.
- `npm run build`: passed with the existing mixed-import chunk warnings only.
- `git diff --check`: passed with line-ending normalization notices only.
- `graphify update .`: rebuilt the code graph with 1505 nodes and 3271 edges.

## Today Workspace

- Added Today as the default top-level workspace while keeping Projects,
  Calendar, Statistics, and Settings mutually exclusive through one typed view
  state. Returning to Projects preserves the selected project.
- Added a parameterized Today summary query that uses explicit local-date
  boundaries, separates incomplete overdue and current-day work, excludes system
  tasks, includes project display metadata, and reports daily completion counts.
- Added a Svelte 5 Today store that concurrently loads tasks, calendar events,
  and time statistics while rejecting stale successes and failures.
- Added responsive Today presentation for the active timer, overdue work, today's
  queue, all-day/timed schedule, task completion progress, and live focused time.
- Added single-column docked/portrait and two-column centered layouts, compact
  mode, reduced-motion behavior, keyboard focus states, loading skeletons,
  retained-snapshot errors, and disconnected-calendar guidance.
- Reused existing task editing, completion, context menu, and timer behavior.
  Shared task mutations, Google Calendar sync, timer changes, task-modal edits,
  window focus, manual refresh, and local midnight now refresh Today.

### Today Verification

- `npm run check`: passed with 0 errors and 0 warnings.
- `npm run test -- --run`: 76 frontend tests passed, including 14 Today store,
  scheduling, formatting, responsive-state, active-timer, and action tests.
- `npm run build`: production SvelteKit build passed; existing dynamic-import
  chunk warnings remain informational.
- `cargo fmt --check` and `cargo check`: passed.
- `cargo test`: 129 Rust tests passed, including 4 Today query tests.
- `git diff --check`: passed.
- `graphify update .`: rebuilt the project graph with 1497 nodes and 3255
  edges; the six reported settings/configuration files are the existing
  non-code sources that produce no AST nodes.

---

## Updater Restart Capability

- Added `process:allow-restart` to the desktop capability so the updater's existing `relaunch()` calls can restart the installed application.
- Kept the capability narrowly scoped: neither `process:default` nor `process:allow-exit` is granted.

### Verification

- Capability JSON parsing and exact process-permission assertion: passed.
- `npm run check`: passed with 0 errors and 0 warnings.
- `cargo check`: passed.
- `git diff --check`: passed.
- `graphify update .`: rebuilt the graph with 1424 nodes and 3105 edges; its expected JSON AST warning included `desktop.json` because capability JSON does not produce code nodes.

---

## Release v0.1.69 macOS Build Recovery

- Corrected the macOS listener imports to resolve away handlers through the
  sibling `system_events` module.
- Preserved the published v0.1.68 tag and partial Windows/Linux assets.
- Added a v0.1.69 release-note entry describing restored Mac availability.

### Pre-release verification

- `cargo fmt -- --check`: passed.
- Host `cargo check`: passed.
- `create-release.ps1 -v 0.1.69 -ValidateOnly`: passed.
- Temporary macOS validation run `31474832657`: passed.
- Full Intel macOS Tauri bundle: passed in 6m04s.
- Full Apple Silicon Tauri bundle: passed in 6m20s.
- `graphify update .`: rebuilt the graph with 1421 nodes and 3102 edges.

### Release verification

- Release commit/tag: `2ef9381` / `v0.1.69`.
- Release workflow `31475400546`: all platform jobs and updater JSON upload passed.
- GitHub release is published as a final release.
- Windows, Linux, Apple Silicon, and Intel macOS manifests all report `0.1.69`
  with notes, publication date, signature, and a matching uploaded artifact.
- The bundled verifier's 20-second CDN checks timed out for two large artifacts;
  direct GitHub release downloads and asset metadata confirmed both are uploaded
  and reachable.

---

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

---

## What's New for Every Update

- Added `src/lib/data/releases.json` as the required release-note catalog,
  starting with version 0.1.65.
- Added numeric version comparison so users who skip versions see every unseen
  release in chronological order.
- Added a main-window What's New modal after update relaunch and a permanent
  Settings entry for reopening release history.
- Stored `whats_new_last_seen_version` in SQLite `app_settings` so dismissal is
  shared across WebView origins.
- Displayed Tauri updater notes before download in both the bottom update banner
  and Settings.
- Updated `create-release.ps1` with catalog validation, Markdown generation,
  package-lock version synchronization, GitHub/updater note publishing, and a
  non-mutating `-ValidateOnly` mode.
- Updated the secondary signing script and GitHub Actions workflow to require
  the same catalog entry and publish the same notes.
- Aligned the updater endpoint with the canonical GitHub repository,
  `PaperPlaneLabs/MyTodos`.
- Added `docs/releasing.md` with the new release procedure.

### What's New Verification

- `npm run check`: 0 errors and 0 warnings.
- `npm run test -- --run`: 62 frontend tests passed.
- `npm run build`: production frontend build passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 123 Rust tests passed, including 3 What's New persistence tests.
- `create-release.ps1 -v 0.1.65 -ValidateOnly`: rendered complete release notes
  without file or network mutation.
- PowerShell syntax validation passed for all three release scripts.
- GitHub Actions workflow YAML parsed successfully with the repository's Node
  YAML dependency.
- `graphify update .`: rebuilt the project graph with 1398 nodes and 2997 edges.

---

## Release v0.1.65 Recovery

- Corrected the Git remote and every release/updater URL to the canonical
  `PaperPlaneLabs/MyTodos` repository after GitHub's old smart-HTTP redirect
  endpoint rejected the push.
- Added `src-tauri/Cargo.lock` to the version files updated by
  `create-release.ps1`.
- Added a GitHub Actions validation gate for the root `my-todos` package version
  in `Cargo.lock`.
- Preserved the existing release notes and recovered the already-created local
  release instead of rerunning the mutation/build flow.

### Recovery Verification

- Confirmed the canonical remote repository with GitHub CLI.
- Confirmed `refs/tags/v0.1.65` did not exist remotely before retagging.
- PowerShell syntax parsing passed for all release scripts.
- `create-release.ps1 -v 0.1.65 -ValidateOnly` passed.
- GitHub Actions workflow YAML parsing passed.
- All six version locations, including the root Cargo lock package, resolve to
  `0.1.65`.
- `git diff --check` passed.
- `graphify update .`: rebuilt the code graph with 1400 nodes and 2999 edges.
  Graphify warned that six settings/configuration sources produced no AST nodes;
  the release paths modified here are present in the refreshed graph.

---

## Project Release Skill

- Added the project-local `release-mytodos` skill for preparing, publishing,
  monitoring, recovering, and verifying MyTodos releases.
- Encoded the canonical `PaperPlaneLabs/MyTodos` repository, the required
  What's New catalog gate, all synchronized version files, and the CI-backed
  `create-release.ps1 -Online` flow.
- Added explicit recovery rules for local-only tags, existing remote tags,
  failed workflows, incomplete updater publication, and obsolete repository
  URLs.
- Added a read-only verifier that checks the tag-triggered Actions run, final
  GitHub release state, four platform-specific updater manifests, their schema,
  versions, signatures, and referenced artifacts.

### Release Skill Verification

- PowerShell syntax parsing passed for `verify-release.ps1`.
- Live verification passed against `v0.1.65` for Windows, Linux, Apple Silicon
  macOS, and Intel macOS.
- A negative verification against unpublished `v9.9.9` exited with failure and
  identified the missing workflow/release plus manifest version mismatches.
- Skill Creator `quick_validate.py` passed in an ephemeral PyYAML environment.
- `git diff --check` passed and no generated TODO placeholders remain.
- `graphify update .`: rebuilt the code graph with 1416 nodes and 3013 edges.

---

## Todoz Stage 1 Rename

- Renamed the user-facing desktop product, main window, header, Settings labels,
  tray action, OAuth completion page, backup folder label, Android display name,
  MCP tool descriptions, and active project documentation to Todoz.
- Preserved `com.pintu.my-todos`, the `my-todos` database directory, Google
  keyring service, Rust/npm package and main binary, `mytodos-mcp` executable and
  client key, updater signing key, release tags, and `PaperPlaneLabs/MyTodos`.
- Changed release titles and local release tooling output to Todoz.
- Reworked platform updater-manifest generation to discover exactly one signed
  asset per platform suffix, allowing Todoz artifact prefixes while rejecting
  ambiguous release assets.
- Kept historical walkthrough entries and legacy path examples unchanged where
  they document real compatibility identities or previous releases.

### Todoz Rename Verification

- Stage 1 compatibility assertions passed for the bundle identifier, database
  directory, keyring service, updater/release repository, npm/Rust package,
  default binary, and MCP server name.
- GitHub Actions YAML parsed with the repository's installed `yaml` package, and
  the embedded updater Bash passed `bash -n`.
- An isolated execution of the exact updater workflow generated all four Todoz
  platform manifests with the correct version, artifact URL, and signature. The
  first run exposed a missing `VERSION` assignment during cleanup; it was
  restored and the simulation then passed.
- PowerShell syntax passed for every modified script.
- `create-release.ps1 -v 0.1.65 -ValidateOnly` passed without mutation or
  network access.
- `npm run check`: 0 errors and 0 warnings.
- `npm run test -- --run`: 62 frontend tests passed.
- `npm run build`: production frontend build passed.
- `cargo fmt -- --check`, `cargo check`, and
  `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 123 Rust tests passed.
- A real signed NSIS build passed and produced
  `Todoz_0.1.65_x64-setup.exe`, its updater zip, and both signatures while the
  compiled application binary remained `my-todos.exe` as required by Stage 1.
- `git diff --check`: passed.
- `graphify update .`: rebuilt the code graph with 1418 nodes and 3015 edges.
  Graphify reported the same six settings/configuration sources that produce no
  AST nodes; the renamed code and release workflow are present in the refreshed
  graph.

---

## Today Workspace Visual Polish

- Refined the landing-page hierarchy by replacing strong nested borders and
  badges with low-contrast surfaces, plain metadata, and subtle elevation.
- Reduced overdue and active-timer emphasis to narrow status accents while
  retaining clear semantic color and timer state visibility.
- Simplified task rows, deadline labels, schedule events, empty states, and
  progress bars; hover and keyboard focus continue to reveal interaction.
- Preserved the existing responsive grid, docked layout, compact mode, theme
  variables, actions, and accessible labels.

### Verification

- `npm run check`: 0 errors and 0 warnings.
- Focused Today Vitest run: 3 files and 9 tests passed.
- `npm run build`: production frontend build passed; only the existing mixed
  dynamic/static import warnings were emitted.
- `git diff --check`: passed (Git reported line-ending normalization notices).
- `graphify update .`: rebuilt the graph with 1497 nodes and 3255 edges.

### Current Timer Task Switching

- The active timer remains visible on Today with its current task, elapsed or
  remaining time, and Pause/Resume and Stop controls.
- Added a Switch task action that opens the standard Projects view through
  `uiStore.openProjectsView()` without pausing or stopping the current timer.
- Added an interaction assertion that Projects navigation occurs and the timer
  stop action is not called.
- `npm run check`: 0 errors and 0 warnings.
- Focused Today workspace tests: 4 tests passed.
- `npm run build`: passed with only the existing mixed-import warnings.
- `git diff --check`: passed with line-ending normalization notices only.
- `graphify update .`: rebuilt the graph with 1499 nodes and 3258 edges.

### Today Timer Visibility Regression

- Compared the current landing-page path with commit `0040a7a`; the active
  timer card was gated behind completion of the independent Today data load.
- Moved the timer card above that loading branch so an active timer remains
  visible even while Today tasks and calendar data show skeletons or fail.
- Added regression coverage for an active timer during initial Today loading,
  including Switch task, Pause, and Stop controls.
- `npm run check`: 0 errors and 0 warnings.
- Focused Today workspace tests: 5 tests passed.
- `npm run build`: passed with only the existing mixed-import warnings.
- `git diff --check`: passed with line-ending normalization notices only.
- `graphify update .`: rebuilt the graph with 1500 nodes and 3259 edges.

### Today Workspace Scrolling

- Replaced the nested Today scroller with a single scroll owner on the
  `100vh` app shell whenever Today is active.
- The Today workspace now grows in normal flow, preventing timer, task,
  schedule, and progress cards from being clipped by the shell's previous
  `overflow: hidden` boundary.
- Added a stable scrollbar gutter to avoid width shifts as the landing page
  grows beyond the window height.
- `npm run check`: 0 errors and 0 warnings.
- Focused Today suite: 3 files and 10 tests passed.
- `npm run build`: passed with only the existing mixed-import warnings.
- `git diff --check`: passed with line-ending normalization notices only.
- `graphify update .`: rebuilt the graph with 1501 nodes and 3260 edges.
