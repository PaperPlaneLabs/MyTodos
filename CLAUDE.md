# MyTodos - Portrait Todo App with Time Tracking

A desktop todo application built with Tauri 2, SvelteKit, and SQLite. Features a portrait-style window optimized for side-of-screen placement with integrated time tracking, window activity monitoring, and Google Calendar integration.

## Tech Stack

- **Frontend**: Svelte 5 + SvelteKit + Vite
- **Backend**: Rust + Tauri 2
- **Database**: SQLite (via rusqlite)
- **Styling**: Custom CSS with CSS variables for theming

## Commands

```bash
# Development
npm install
npm run tauri dev          # Full app (frontend + Rust backend)
npm run dev                # Frontend only (no Tauri)

# Type checking
npm run check              # svelte-check + tsc

# Testing
npm test                   # Vitest (frontend unit tests)
cargo test --manifest-path src-tauri/Cargo.toml  # Rust integration tests

# Build
npm run tauri build        # Production bundle → src-tauri/target/release/bundle/
```

## Project Structure

```
MyTodos/
├── src/                          # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── calendar/        # CalendarTabView, CalendarMonth, CalendarWeek, etc.
│   │   │   ├── common/          # Modal, BreakView, ContextMenu, TimeDisplay, etc.
│   │   │   ├── layout/          # AppHeader, CollapseHandle
│   │   │   ├── modals/          # PageModalHost
│   │   │   ├── projects/        # ProjectListSection
│   │   │   ├── resume/          # ResumeView
│   │   │   ├── settings/        # SettingsView
│   │   │   ├── stats/           # StatsView
│   │   │   ├── tasks/           # TaskListSection
│   │   │   └── timer/           # ActiveTimerWidget
│   │   ├── controllers/
│   │   │   └── page-interactions.svelte.ts  # Cross-component interaction logic
│   │   ├── services/
│   │   │   ├── db.ts            # Tauri command wrappers (all invoke() calls)
│   │   │   └── time-parser.ts   # Time string parsing utilities
│   │   ├── stores/
│   │   │   ├── afk-categories.svelte.ts
│   │   │   ├── calendar.svelte.ts
│   │   │   ├── google-calendar.svelte.ts
│   │   │   ├── projects.svelte.ts
│   │   │   ├── tasks.svelte.ts
│   │   │   ├── timer.svelte.ts
│   │   │   ├── timer-break-reminders.svelte.ts
│   │   │   ├── timer-events.ts
│   │   │   ├── timer-runtime.svelte.ts
│   │   │   ├── ui.svelte.ts
│   │   │   └── window-tracking.svelte.ts
│   │   ├── styles/
│   │   │   ├── global.css
│   │   │   └── theme.css
│   │   ├── test-utils/
│   │   │   └── tauri-mock.ts    # Vitest mock for invoke()
│   │   └── types/
│   │       └── calendar.ts
│   └── routes/
│       ├── +layout.js           # CSR config
│       └── +page.svelte         # Main app
│
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── app/                 # App lifecycle
│   │   │   ├── startup.rs       # Initialization sequence
│   │   │   ├── tray.rs          # System tray menu
│   │   │   └── window_lifecycle.rs  # Close/minimize behavior
│   │   ├── commands/            # Tauri IPC commands
│   │   │   ├── calendar.rs
│   │   │   ├── google_calendar.rs
│   │   │   ├── projects.rs
│   │   │   ├── sections.rs
│   │   │   ├── tasks.rs
│   │   │   ├── time_entries.rs
│   │   │   ├── time_stats.rs
│   │   │   ├── timer.rs
│   │   │   ├── window.rs
│   │   │   └── window_tracking.rs
│   │   ├── db/                  # Database layer
│   │   │   ├── connection.rs
│   │   │   ├── models.rs
│   │   │   └── schema.rs        # Schema + migrations
│   │   ├── events/              # OS-level event handling (shutdown detection)
│   │   │   ├── linux.rs
│   │   │   ├── macos.rs
│   │   │   ├── windows.rs
│   │   │   └── system_events.rs
│   │   ├── google/              # Google Calendar integration
│   │   │   ├── calendar_api.rs
│   │   │   ├── oauth.rs
│   │   │   ├── sync.rs
│   │   │   └── token_store.rs
│   │   ├── services/
│   │   │   ├── mod.rs           # Shared constants (system project names/colors)
│   │   │   ├── timer_service.rs
│   │   │   └── window_tracking_service.rs
│   │   ├── error.rs             # Custom error types with Serde serialization
│   │   └── lib.rs               # Tauri app initialization + command registration
│   ├── tests/                   # Integration tests (run with cargo test)
│   └── Cargo.toml
│
└── package.json
```

## Database Schema

### Tables

**projects / tasks / sections** — core hierarchy. Both `projects` and `tasks` have `is_system BOOLEAN` for auto-generated system entries (AFK, Breaks).

**time_entries** — timer or manual records; `entry_type IN ('timer', 'manual')`

**active_timer** (singleton, `id = 1`) — running timer state with pause/resume support

**window_state** (singleton, `id = 1`) — persisted window position and dock preference (`left`/`center`/`right`)

**app_settings** — key-value store for feature flags (e.g., `window_tracking_enabled`)

**window_activity_entries** — foreground app segments; polled every 5 s by `window_tracking_service`

**active_window_tracking** (singleton, `id = 1`) — currently-active foreground app segment

**calendar_events** — local calendar events (separate from Google Calendar sync)

### Key Schema Features

- **Cascading Deletes**: `ON DELETE CASCADE` on all foreign keys
- **Denormalized Time**: `total_time_seconds` cached on projects/tasks for fast reads
- **Migrations**: Additive-only `ALTER TABLE` at end of `schema.rs` — never destructive

## Core Concepts

### Svelte 5 Runes-Based Stores

```typescript
let projects = $state<Project[]>([]);

export const projectStore = {
  get projects() { return projects; },
  get selected() { return projects.find(p => p.id === selectedProjectId) ?? null; },
  async loadAll() { /* invoke + assign */ }
};
```

All stores live in `src/lib/stores/`. Components access via `$derived`:

```svelte
<script>
  import { projectStore } from '$lib/stores/projects.svelte';
  let currentProject = $derived(projectStore.selected);
</script>
```

### Tauri Command Pattern

```typescript
// src/lib/services/db.ts — all invoke() calls centralized here
const project = await invoke<Project>('create_project', { name: 'My Project' });
```

```rust
// src-tauri/src/commands/projects.rs
#[tauri::command]
pub fn create_project(db: State<DbConnection>, name: String) -> Result<Project> { ... }
// Register in lib.rs generate_handler![]
```

### Timer State Management

Client-side interval for smooth UI; backend is source of truth:

- `start_timer(task_id)` → inserts `active_timer`, starts interval
- `pause_timer()` → updates `elapsed_seconds`, stops interval
- `resume_timer()` → updates `started_at`, restarts interval
- `stop_timer()` → creates `time_entry`, updates totals, deletes `active_timer`

```
elapsed = elapsed_seconds + (now - started_at)
```

### Window Tracking

`window_tracking_service` polls foreground app every 5 s and writes `window_activity_entries`. Controlled via `app_settings` key `window_tracking_enabled`. AFK time (Away/Breaks system projects) is merged into stats alongside app activity; entries carry `kind: "app" | "afk"`.

### Theme System

```css
:root { --bg-primary: #ffffff; --accent: #6366f1; }
[data-theme="dark"] { --bg-primary: #1a1a1a; --accent: #818cf8; }
```

Toggle via `uiStore.toggleTheme()` → sets `data-theme` on `<body>`.

## Google Calendar Integration

Requires OAuth credentials. Copy `.env.example` → `.env` and fill in:

```
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret
```

These are baked into the binary at build time via Tauri's build script. The `src-tauri/src/google/` module handles OAuth flow, token refresh, and Calendar API sync.

## Adding a New Tauri Command

1. Add function with `#[tauri::command]` in the appropriate `src-tauri/src/commands/*.rs` file
2. Register it in `lib.rs` → `generate_handler![]`
3. Add `invoke<T>()` wrapper in `src/lib/services/db.ts`
4. Create or update the relevant store in `src/lib/stores/`

## Database Location

- **Windows**: `%APPDATA%\my-todos\todos.db`
- **macOS**: `~/Library/Application Support/my-todos/todos.db`
- **Linux**: `~/.local/share/my-todos/todos.db`

## Debugging

- **Rust**: logs in terminal running `npm run tauri dev`; use `println!` / `eprintln!`
- **Frontend**: F12 DevTools in app window
- **Database**: inspect with any SQLite browser; `PRAGMA foreign_keys;` to verify FK enforcement

## Known Limitations

- Single active timer at a time (by design, enforced via `id = 1` constraint)
- Single window only
- No cloud sync (Google Calendar is read/write but todos stay local)
