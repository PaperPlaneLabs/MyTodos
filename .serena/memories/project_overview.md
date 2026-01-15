# MyTodos Project Overview

## Purpose
A desktop todo application with time tracking, built with Tauri 2, SvelteKit, and SQLite. Features portrait-style window for side-of-screen placement.

## Tech Stack
- **Frontend**: Svelte 5 (runes-based), SvelteKit, Vite, TypeScript
- **Backend**: Rust, Tauri 2, SQLite (rusqlite)
- **Database**: SQLite with cascading deletes and foreign keys
- **Styling**: Custom CSS with CSS variables for theming

## Key Architecture Patterns
- **State Management**: Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Tauri IPC**: Commands with `#[tauri::command]` macro
- **Service Layer**: Tauri command wrappers in `src/lib/services/db.ts`
- **Database Connection**: Managed via `Arc<Mutex<Connection>>` in Rust
- **Error Handling**: Custom error types with serde serialization

## Code Style
- **Rust**: Edition 2021, no particular style enforced yet
- **TypeScript**: Typed interfaces for all data structures
- **Svelte**: Runes-based reactivity, component-driven architecture
- **Naming**: snake_case for Rust, camelCase for TypeScript

## Database Schema
Tables: projects, sections, tasks, time_entries, active_timer, window_state
- Flexible hierarchy: tasks can belong to sections or projects
- Denormalized time tracking via total_time_seconds
- Position fields for drag-and-drop reordering
