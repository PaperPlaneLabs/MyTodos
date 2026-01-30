# Agent Documentation for MyTodos

A desktop todo application built with Tauri 2, SvelteKit, and SQLite. This document provides guidance for AI assistants working on this codebase.

## Project Overview

### Tech Stack
- **Frontend**: Svelte 5 + SvelteKit + Vite
- **Backend**: Rust + Tauri 2
- **Database**: SQLite (via rusqlite)
- **Styling**: Custom CSS with CSS variables for theming

### Key Features
- Portrait-optimized window (default ~380px width)
- Integrated time tracking with timer
- Project and task management
- Statistics view with time visualizations
- Calendar view with deadline management
- Multiple themes (light, dark, minecraft, retro, ocean, nord)
- Window docking (left/right)
- Collapsible app state

## Architecture

### Directory Structure

```
MyTodos/
├── src/                          # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── calendar/         # Calendar components
│   │   │   ├── common/           # Reusable UI components
│   │   │   ├── layout/           # Layout components
│   │   │   ├── settings/         # Settings view
│   │   │   └── stats/            # Statistics view
│   │   ├── services/
│   │   │   ├── db.ts             # Tauri command wrappers
│   │   │   └── time-parser.ts    # Time parsing utilities
│   │   ├── stores/
│   │   │   ├── calendar.svelte.ts # Calendar state
│   │   │   ├── projects.svelte.ts # Project state
│   │   │   ├── tasks.svelte.ts    # Task state
│   │   │   ├── timer.svelte.ts    # Timer state
│   │   │   └── ui.svelte.ts       # UI state
│   │   ├── styles/
│   │   │   ├── global.css         # Global styles
│   │   │   └── theme.css          # CSS variables & themes
│   │   └── types/
│   │       └── calendar.ts        # Calendar TypeScript types
│   └── routes/
│       └── +page.svelte           # Main app page
│
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── commands/             # Tauri commands
│   │   │   ├── calendar.rs       # Calendar-related commands
│   │   │   ├── mod.rs            # Command exports
│   │   │   ├── projects.rs       # Project CRUD
│   │   │   ├── sections.rs       # Section management
│   │   │   ├── tasks.rs          # Task CRUD
│   │   │   ├── time_entries.rs   # Time tracking
│   │   │   ├── time_stats.rs     # Statistics
│   │   │   ├── timer.rs          # Timer operations
│   │   │   └── window.rs         # Window management
│   │   ├── db/
│   │   │   ├── connection.rs     # DB connection management
│   │   │   ├── models.rs         # Data models
│   │   │   └── schema.rs         # Database schema
│   │   ├── error.rs              # Error types
│   │   └── lib.rs                # Tauri app initialization
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── package.json
└── vite.config.js
```

### State Management (Svelte 5 Runes)

The frontend uses Svelte 5's runes (`$state`, `$derived`, `$effect`) for reactivity. Stores are exported as objects with getter methods:

```typescript
// Example store pattern
let items = $state<Item[]>([]);

export const itemStore = {
    get items() { return items; },
    get activeItems() { return items.filter(i => !i.completed); },
    
    async loadAll() { /* ... */ },
    async create(item: Item) { /* ... */ },
    async update(id: number, data: Partial<Item>) { /* ... */ },
};
```

### Database Schema

**Tables:**
- `projects` - Top-level project containers with color and position
- `sections` - Optional subdivisions within projects
- `tasks` - Individual todo items with deadline support
- `time_entries` - Time tracking records
- `active_timer` - Singleton for running timer state
- `window_state` - Singleton for window position/size
- `calendar_events` - All-day calendar events

**Key Columns:**
- `tasks.deadline` - ISO date string (YYYY-MM-DD) for task deadlines
- `tasks.position` - For drag-and-drop ordering
- `tasks.completed` - Boolean completion status

### Tauri Command Pattern

Frontend calls backend via `invoke`:

```typescript
// TypeScript (frontend)
const result = await db.tasks.getByProject(projectId);

// Rust (backend)
#[tauri::command]
pub fn get_tasks_by_project(db: State<DbConnection>, project_id: i64) -> Result<Vec<Task>> {
    let conn = db.lock();
    // ... query
}
```

## Common Tasks

### Adding a New Tauri Command

1. **Create the command** in `src-tauri/src/commands/<domain>.rs`:
```rust
#[tauri::command]
pub fn my_command(db: State<DbConnection>, param: String) -> Result<MyResult> {
    let conn = db.lock();
    // implementation
}
```

2. **Export from mod.rs**:
```rust
pub mod domain;
pub use domain::*;
```

3. **Register in lib.rs**:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing
    commands::my_command,
])
```

4. **Add TypeScript wrapper** in `src/lib/services/db.ts`:
```typescript
myCommand: (param: string) => invoke<MyResult>("my_command", { param }),
```

### Adding a New Store

1. Create `src/lib/stores/<name>.svelte.ts`
2. Use Svelte 5 runes for state
3. Export as an object with getter methods
4. Import and use in components

### Modifying the Database Schema

1. **Update schema.rs** - Add `CREATE TABLE` or `ALTER TABLE` statements
2. **Update models.rs** - Add/modify struct with `#[derive(Debug, Clone, Serialize, Deserialize)]`
3. **Update db.ts** - Add TypeScript interface
4. **Create migration** - Add `ALTER TABLE` commands that can run on existing databases

### Adding a Calendar Event Type

1. **Update schema.rs** - Add to `calendar_events` table if needed
2. **Update models.rs (Rust)** - Add model struct
3. **Update db.ts (TypeScript)** - Add interface
4. **Create commands** in `commands/calendar.rs`
5. **Create calendar store** methods in `calendar.svelte.ts`
6. **Create UI components** in `src/lib/components/calendar/`

### Adding a New Theme

Edit `src/lib/styles/theme.css`:
```css
[data-theme="themename"] {
    --bg-primary: #ffffff;
    --text-primary: #1a1a1a;
    --accent: #6366f1;
    /* ... other variables */
}
```

Add to the theme switcher in SettingsView.

## Important Patterns

### Modal Management

Modals are controlled via `uiStore`:
- `showTaskModal`, `showProjectModal`, etc. - boolean flags
- `editingTaskId`, `editingProjectId` - ID of item being edited
- `openTaskModal({ taskId?, deadline? })` - Open with optional prefill

### Date Handling

- Store dates as ISO strings (YYYY-MM-DD) in SQLite
- Use JavaScript `Date` object in frontend
- Use `chrono::Utc::now().timestamp()` in Rust for timestamps

### Timer State

The timer uses:
- Client-side interval for smooth UI updates
- Backend sync via Tauri commands
- `active_timer` singleton table for persistence

### Drag and Drop

Uses pointer events for cross-platform support:
- `onpointerdown` - Start drag
- `onpointermove` - Track position
- `onpointerup` - End drag
- `ondragstart`, `ondrop` - For HTML5 drag API (calendar)

## Development Commands

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build

# Run linting (if configured)
npm run lint

# Type checking (if configured)
npm run typecheck
```

## Database Location

- **Windows**: `%APPDATA%\my-todos\todos.db`
- **macOS**: `~/Library/Application Support/my-todos/todos.db`
- **Linux**: `~/.local/share/my-todos/todos.db`

## Key Files Reference

| File | Purpose |
|------|---------|
| `+page.svelte` | Main app layout, task list, modals |
| `AppHeader.svelte` | Header with stats, settings, calendar |
| `tasks.svelte.ts` | Task state and CRUD operations |
| `calendar.svelte.ts` | Calendar state and date management |
| `calendar.rs` | Backend calendar commands |
| `tasks.rs` | Backend task commands |
| `schema.rs` | Database schema and migrations |

## Common Issues & Solutions

### Schema Migrations Failing
- Use `ALTER TABLE ADD COLUMN` for new columns
- Create indexes separately after table creation
- Use `let _ = conn.execute(...)` to ignore errors if column exists

### TypeScript/Rust Type Mismatch
- Ensure Rust models derive `Serialize, Deserialize`
- Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- Match TypeScript interface field names to Rust struct fields

### Calendar Performance
- Load tasks for month on calendar open
- Use `$derived` for calendar day computation
- Consider lazy loading for large datasets

## Testing

To test changes:
1. Run `npm run tauri dev` for development
2. Create/modify tasks with deadlines
3. Open calendar and verify tasks appear on correct dates
4. Test drag-and-drop rescheduling
5. Test deadline picker in task modal

## Code Style

- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Avoid `$:` reactive statements
- Use `$props()` for component props
- Follow existing CSS variable naming (`--spacing-md`, `--text-sm`, etc.)
- No comments unless explaining complex logic
