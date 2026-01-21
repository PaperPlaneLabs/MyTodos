# MyTodos - Project Context & Instructions

This file provides a comprehensive overview of the **MyTodos** project to guide Gemini in future interactions.

## Project Overview

**MyTodos** is a portrait-style desktop todo application with integrated time tracking. It is designed to be placed on the side of the screen.

- **Frontend:** Svelte 5 (Runes-based) + SvelteKit + Vite
- **Backend:** Rust + Tauri 2
- **Database:** SQLite (via `rusqlite`)
- **State Management:** Svelte 5 Runes (`$state`, `$derived`, `$effect`)
- **Target Platforms:** Windows, macOS, Linux (with initial scaffolding for Android)

## Project Structure

```
MyTodos/
├── src/                          # Frontend (SvelteKit + Svelte 5)
│   ├── lib/
│   │   ├── components/          # UI Components
│   │   │   ├── common/          # Reusable UI components (Modal, etc.)
│   │   │   ├── layout/          # Layout components (AppHeader, etc.)
│   │   │   ├── settings/        # Settings views
│   │   │   └── stats/           # Statistics views
│   │   ├── services/            # API/Database service wrappers (db.ts)
│   │   ├── stores/              # Svelte 5 Runes-based stores
│   │   │   ├── projects.svelte.ts
│   │   │   ├── tasks.svelte.ts
│   │   │   ├── timer.svelte.ts
│   │   │   └── ui.svelte.ts
│   │   └── styles/              # Global and theme CSS
│   └── routes/                  # App routes (+page.svelte is the main entry)
├── src-tauri/                    # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── commands/            # Tauri IPC command implementations
│   │   │   ├── mod.rs           # Command module exports
│   │   │   ├── projects.rs      # Project management
│   │   │   ├── sections.rs      # Section management
│   │   │   ├── tasks.rs         # Task management
│   │   │   ├── time_entries.rs  # Time entry management
│   │   │   ├── time_stats.rs    # Statistics calculation
│   │   │   ├── timer.rs         # Active timer logic
│   │   │   └── window.rs        # Window state persistence
│   │   ├── db/                  # SQLite database layer
│   │   │   ├── connection.rs
│   │   │   ├── models.rs
│   │   │   └── schema.rs        # Database schema definitions
│   │   ├── error.rs             # Custom Error handling
│   │   ├── lib.rs               # App initialization and command registration
│   │   └── main.rs              # Entry point
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── package.json                 # Node dependencies and scripts
└── CLAUDE.md                    # Additional development guidelines
```

## Building and Running

### Development
```bash
# Install dependencies
npm install

# Run the app in development mode
npm run tauri dev
```

### Build & Check
```bash
# Type check and Svelte check
npm run check

# Build for production
npm run tauri build
```

## Development Conventions

### Frontend (Svelte 5)
- **Runes:** Use `$state` for reactive variables, `$derived` for computed values, and `$effect` for side effects.
- **Stores:** Organize logic into stores in `src/lib/stores/*.svelte.ts`. These should expose getters and methods for state mutation.
- **IPC:** All backend communication should go through `src/lib/services/db.ts` using Tauri's `invoke`.

### Backend (Rust)
- **Commands:** Group commands by domain (e.g., `projects.rs`, `tasks.rs`) in `src-tauri/src/commands/`.
- **Database:** Use parameterized queries to prevent SQL injection. Maintain the schema in `src-tauri/src/db/schema.rs`.
- **Error Handling:** Use the custom `AppError` type and `Result<T>` alias defined in `src-tauri/src/error.rs`.
- **State:** The database connection is managed as Tauri state (`State<DbConnection>`).

### Database Schema
- **Flexible Hierarchy:** Tasks can belong to a project directly or to a section within a project.
- **Time Tracking:** Time is tracked via `time_entries` and a singleton `active_timer` table for the currently running timer.
- **Window State:** `window_state` table (singleton) persists window position and dimensions.
- **Denormalization:** `total_time_seconds` is cached on projects, sections, and tasks for performance.

## Key Files to Reference
- `CLAUDE.md`: Contains detailed architectural patterns and feature implementation guides.
- `src/lib/services/db.ts`: The bridge between frontend and backend.
- `src-tauri/src/db/schema.rs`: The source of truth for the database structure.
- `src-tauri/src/lib.rs`: Registration point for all Tauri commands.