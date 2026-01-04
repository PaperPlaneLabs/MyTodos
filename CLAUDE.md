# MyTodos - Portrait Todo App with Time Tracking

A desktop todo application built with Tauri 2, SvelteKit, and SQLite. Features a portrait-style window optimized for side-of-screen placement with integrated time tracking.

## Tech Stack

- **Frontend**: Svelte 5 + SvelteKit + Vite
- **Backend**: Rust + Tauri 2
- **Database**: SQLite (via rusqlite)
- **Styling**: Custom CSS with CSS variables for theming

## Architecture Overview

### Frontend (SvelteKit)

The frontend is a single-page application using Svelte 5's runes-based reactivity system. It communicates with the Rust backend via Tauri's IPC (Inter-Process Communication).

**Key Patterns:**
- **State Management**: Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Store Pattern**: Reactive stores with getters and async methods
- **Service Layer**: Tauri command wrappers in `src/lib/services/db.ts`
- **Component Architecture**: Presentational components with smart containers

### Backend (Rust + Tauri)

The backend handles all data persistence and business logic using SQLite.

**Key Patterns:**
- **Command-Based API**: Tauri commands for frontend-backend communication
- **Shared State**: Database connection wrapped in `Arc<Mutex<Connection>>`
- **Error Handling**: Custom error types with Serde serialization
- **Modular Commands**: Separate command modules for each domain (projects, tasks, timer, etc.)

## Project Structure

```
MyTodos/
├── src/                          # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── common/          # Reusable UI components
│   │   │   │   ├── Modal.svelte
│   │   │   │   └── TimeDisplay.svelte
│   │   │   └── layout/          # Layout components
│   │   │       └── AppHeader.svelte
│   │   ├── services/
│   │   │   ├── db.ts            # Tauri command wrappers
│   │   │   └── time-parser.ts  # Time parsing utilities
│   │   ├── stores/
│   │   │   ├── projects.svelte.ts
│   │   │   ├── tasks.svelte.ts
│   │   │   ├── timer.svelte.ts
│   │   │   └── ui.svelte.ts
│   │   └── styles/
│   │       ├── global.css
│   │       └── theme.css
│   └── routes/
│       ├── +layout.js           # CSR config
│       └── +page.svelte         # Main app
│
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── commands/            # Tauri commands
│   │   │   ├── mod.rs
│   │   │   ├── projects.rs
│   │   │   ├── sections.rs
│   │   │   ├── tasks.rs
│   │   │   ├── timer.rs
│   │   │   ├── time_entries.rs
│   │   │   └── window.rs
│   │   ├── db/                  # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs
│   │   │   ├── models.rs
│   │   │   └── schema.rs
│   │   ├── error.rs             # Custom error types
│   │   └── lib.rs               # Tauri app initialization
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── package.json
└── vite.config.js
```

## Database Schema

### Tables

**projects**
- Stores top-level project containers
- Has color coding and position for ordering
- Caches total time spent (`total_time_seconds`)

**sections** (optional)
- Subdivisions within projects
- Allows flexible 2-3 level hierarchy
- Tasks can belong to sections or directly to projects

**tasks**
- Individual todo items
- `section_id` is nullable (flexible hierarchy)
- Tracks completion status and time spent

**time_entries**
- Records of time spent (timer or manual)
- Links to tasks via foreign key
- Stores start/end times for timer entries

**active_timer** (singleton)
- Only one active timer at a time (enforced by `id = 1` constraint)
- Stores running timer state with pause/resume support

**window_state** (singleton)
- Persists window position and size between sessions

### Key Schema Features

1. **Flexible Hierarchy**: Tasks can have `section_id = NULL` to belong directly to projects
2. **Cascading Deletes**: Foreign keys with `ON DELETE CASCADE`
3. **Denormalized Time**: `total_time_seconds` cached for performance
4. **Position Fields**: Enable drag-and-drop reordering

## Core Concepts

### 1. Svelte 5 Runes-Based Stores

Stores use Svelte 5's runes for reactivity:

```typescript
let projects = $state<Project[]>([]);
let selectedProjectId = $state<number | null>(null);

export const projectStore = {
  get projects() { return projects; },
  get selected() {
    return projects.find(p => p.id === selectedProjectId) ?? null;
  },
  async loadAll() { /* ... */ }
};
```

**Usage in components:**
```svelte
<script>
  import { projectStore } from '$lib/stores/projects.svelte';

  // Reactive access
  let currentProject = $derived(projectStore.selected);
</script>
```

### 2. Timer State Management

The timer uses client-side intervals for smooth UI updates while syncing with the backend:

- `start_timer(task_id)`: Creates active_timer record, starts interval
- `pause_timer()`: Updates elapsed time, stops interval
- `resume_timer()`: Updates started_at, restarts interval
- `stop_timer()`: Creates time_entry, updates totals, deletes active_timer

**Elapsed Time Calculation:**
```
elapsed = elapsed_seconds + (now - started_at)
```

### 3. Tauri Command Pattern

Frontend calls backend via Tauri's `invoke`:

```typescript
// Frontend (TypeScript)
const project = await invoke<Project>('create_project', {
  name: 'My Project'
});

// Backend (Rust)
#[tauri::command]
pub fn create_project(
    db: State<DbConnection>,
    name: String
) -> Result<Project> {
    // Implementation
}
```

### 4. Theme System

CSS variables enable dynamic theming:

```css
:root {
  --bg-primary: #ffffff;
  --text-primary: #1a1a1a;
  --accent: #6366f1;
}

[data-theme="dark"] {
  --bg-primary: #1a1a1a;
  --text-primary: #f5f5f5;
  --accent: #818cf8;
}
```

Theme toggled via `uiStore.toggleTheme()` which updates `data-theme` attribute.

## Development Workflow

### Setup

```bash
# Install dependencies
npm install

# Run development server
npm run tauri dev
```

### Adding a New Feature

**Example: Adding Task Tags**

1. **Update Database Schema** (`src-tauri/src/db/schema.rs`)
   ```sql
   CREATE TABLE tags (
       id INTEGER PRIMARY KEY,
       task_id INTEGER,
       name TEXT,
       FOREIGN KEY (task_id) REFERENCES tasks(id)
   );
   ```

2. **Create Model** (`src-tauri/src/db/models.rs`)
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct Tag {
       pub id: i64,
       pub task_id: i64,
       pub name: String,
   }
   ```

3. **Add Commands** (`src-tauri/src/commands/tags.rs`)
   ```rust
   #[tauri::command]
   pub fn create_tag(db: State<DbConnection>, task_id: i64, name: String) -> Result<Tag> {
       // Implementation
   }
   ```

4. **Register Commands** (`src-tauri/src/lib.rs`)
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       commands::create_tag,
   ])
   ```

5. **Add Service Wrapper** (`src/lib/services/db.ts`)
   ```typescript
   tags: {
     create: (taskId: number, name: string) =>
       invoke<Tag>('create_tag', { taskId, name }),
   }
   ```

6. **Create Store** (`src/lib/stores/tags.svelte.ts`)
   ```typescript
   let tags = $state<Tag[]>([]);
   export const tagStore = {
     get tags() { return tags; },
     async create(taskId: number, name: string) { /* ... */ }
   };
   ```

7. **Build UI Components** (`src/lib/components/tags/`)

## Common Tasks

### Update Window Size

Edit `src-tauri/tauri.conf.json`:
```json
{
  "app": {
    "windows": [{
      "width": 400,
      "height": 850
    }]
  }
}
```

### Add New Theme Colors

Edit `src/lib/styles/theme.css`:
```css
:root {
  --custom-color: #abcdef;
}

[data-theme="dark"] {
  --custom-color: #fedcba;
}
```

### Create New Tauri Command

1. Create function in appropriate command file
2. Add `#[tauri::command]` attribute
3. Register in `lib.rs`'s `generate_handler!` macro
4. Add TypeScript wrapper in `db.ts`

## Database Location

- **Windows**: `%APPDATA%\my-todos\todos.db`
- **macOS**: `~/Library/Application Support/my-todos/todos.db`
- **Linux**: `~/.local/share/my-todos/todos.db`

## Building for Production

```bash
# Build the app
npm run tauri build

# Output location:
# src-tauri/target/release/bundle/
```

## Debugging

### Rust Backend
- Logs appear in terminal running `npm run tauri dev`
- Add `println!` or `dbg!` macros for debugging

### Frontend
- Open DevTools in the app window (F12)
- Console logs appear in DevTools
- Use Svelte DevTools extension for component inspection

### Database
- Use SQLite browser to inspect `todos.db`
- Check foreign key constraints: `PRAGMA foreign_keys;`

## Known Limitations

1. **Single Active Timer**: Only one timer can run at a time (by design)
2. **No Multi-Window**: Single window application
3. **Local Only**: No cloud sync (future enhancement)
4. **Basic Error Handling**: Errors shown in console, minimal user feedback

## Future Enhancements

- [ ] Drag-and-drop reordering (UI + backend ready)
- [ ] Manual time entry UI
- [ ] Project/task edit and delete UI
- [ ] Section management UI
- [ ] Recurring tasks
- [ ] Task dependencies
- [ ] Export/import (JSON, CSV)
- [ ] Cloud sync
- [ ] Keyboard shortcuts
- [ ] Rich text task notes
- [ ] Weekly/monthly reports
- [ ] Pomodoro mode

## Performance Considerations

1. **Cached Aggregates**: Time totals are denormalized for fast reads
2. **Indexed Queries**: Foreign key columns have indexes
3. **Minimal Re-renders**: Svelte 5 fine-grained reactivity
4. **Client-Side Timer**: Reduces backend calls during active timing

## Security Notes

- SQLite injection prevented via parameterized queries
- Foreign keys enforced at database level
- No external network calls (fully offline)
- User data stays local (no telemetry)

## License

MIT

## Author

Built with Claude Code
