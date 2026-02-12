# Agent Development Guidelines - MyTodos

This document provides essential information for AI agents working on the MyTodos codebase.

## Build, Lint, and Test Commands

### Frontend (SvelteKit)
- **Install Dependencies**: `npm install`
- **Development**: `npm run dev`
- **Build**: `npm run build`
- **Type Check**: `npm run check`
- **Run All Tests**: `npm run test`
- **Run Single Test File**: `npx vitest src/lib/services/time-parser.test.ts`
- **Run Tests with UI**: `npm run test:ui`

### Backend (Rust / Tauri)
- **Development (with UI)**: `npm run tauri dev`
- **Build Release**: `npm run tauri build`
- **Lint (Clippy)**: `cargo clippy --all-targets -- -D warnings`
- **Format**: `cargo fmt`
- **Run All Tests**: `cargo test`
- **Run Single Test**: `cargo test test_name`
- **Check (No Build)**: `cargo check` (Run this inside `src-tauri` directory)

---

## Code Style & Conventions

### Frontend (TypeScript & Svelte 5)

- **Svelte 5 Runes**: Always use `$state`, `$derived`, and `$effect`. Avoid Svelte 4 stores (`writable`, `derived`) unless necessary for legacy compatibility.
- **Store Pattern**: Use the object-based store pattern with getters for reactivity.
  ```typescript
  let items = $state<Item[]>([]);
  export const itemStore = {
    get items() { return items; },
    async load() { items = await db.getItems(); }
  };
  ```
- **Imports**: 
  - Use `$lib/` alias for all internal imports (e.g., `import { db } from "$lib/services/db"`).
  - Order: External libraries, `$lib/...`, then relative imports.
- **Naming**:
  - Components: `PascalCase` (e.g., `TaskItem.svelte`).
  - Variables/Functions: `camelCase`.
  - Interfaces/Types: `PascalCase`.
- **Typing**: Prefer explicit interfaces over `any`. Define shared models in `src/lib/services/db.ts` or `src/lib/types/`.
- **Error Handling**: Wrap async service calls in `try/catch` within stores. Log errors and update a reactive `error` state.
- **Compact Mode**: Use the global `compact-mode` class on `<body>` for density adjustments. Check `uiStore.compactMode`.

### Backend (Rust)

- **Error Handling**: Use the custom `AppError` and `Result<T>` defined in `src-tauri/src/error.rs`.
  - Prefer the `?` operator for propagation.
  - Use `anyhow` for complex error context if needed.
- **Tauri Commands**: 
  - Define commands in `src-tauri/src/commands/`.
  - Use `snake_case` for function names.
  - Return `Result<T>`.
  - Use `State<DbConnection>` for database access.
  - **Register New Commands**: Add to `tauri::generate_handler![]` in `src-tauri/src/lib.rs`.
- **Numeric Types**: When defining float variables (e.g., for window size), ALWAYS explicitly specify the type `f64` to avoid ambiguous type errors (e.g., `let width: f64 = 1000.0;`).
- **Database (SQLite)**:
  - Use parameterized queries to prevent SQL injection.
  - Acquire database locks using `db.lock()`.
  - Follow the schema defined in `src-tauri/src/db/schema.rs`.
- **Naming**: Follow standard Rust conventions (`snake_case` for functions/variables, `PascalCase` for structs/enums).
- **Async**: Use `tauri::async_runtime::spawn` for fire-and-forget background tasks (like syncing).

---

## Project Architecture

- **Communication**: Frontend calls backend via `invoke<T>('command_name', { args })` wrapped in `src/lib/services/db.ts`.
- **Persistence**: SQLite database managed by Rust. Time totals are denormalized in projects/sections for performance.
- **Styling**: Global CSS variables in `theme.css`. Use `data-theme="dark"` on the `html` element for theming.
- **Window Management**: Custom window controls (Minimize, Maximize, Close, Dock Left/Right/Center) are handled in `src-tauri/src/commands/window.rs`.
  - "FreeForm" centers the window.
  - "Dock" snaps to sides.
- **State Management**: 
  - `uiStore` handles UI state (modals, theme, compact mode, window orientation).
  - `projectStore` / `taskStore` handle data.

## Debugging & Common Issues

- **Rust Compilation**: If you see "ambiguous numeric type" errors for floats, add explicit `: f64` type annotations.
- **Svelte Check**: Run `npm run check` to validate Svelte code. Note that IDE/LSP errors might be stale; trust `svelte-check`.
- **Database Migrations**: We currently use a simple schema initialization. Changes to `schema.rs` may require manual database resets during development. A "Reset Application" button exists in Settings -> Data Management to clear all projects/tasks.
- **Secrets**: NEVER commit API keys or tokens. Use `keyring` for sensitive data storage (already integrated for Google OAuth).
- **Testing**: When adding features, add Vitest tests for frontend logic and Rust `#[test]` for backend logic.
