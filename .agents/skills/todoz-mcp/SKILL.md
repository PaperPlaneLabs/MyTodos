---
name: todoz-mcp
description: Guide for integrating, testing, and developing with the Todoz Model Context Protocol (MCP) server. Use when querying or modifying tasks, projects, time tracking, hours, deadlines, timers, or when maintaining the todoz-mcp binary.
---

# Todoz MCP Server Integration & Development

This skill provides comprehensive instructions for interacting with, testing, and extending the **Todoz MCP Server** (`todoz-mcp`).

## 1. Overview

Todoz includes a local stdio Model Context Protocol (MCP) server connecting directly to the local SQLite database (`todos.db`) and Google Calendar integration. It allows AI clients (Antigravity, Gemini CLI, Claude Code, Claude Desktop) to interact with projects, tasks, deadlines, and time tracking.

- **Binary Source**: `src-tauri/src/bin/todoz-mcp.rs`
- **Compiled Executable**: `src-tauri/target/release/todoz-mcp.exe`
- **Documentation**: `docs/todoz-mcp.md`
- **Integration Tests**: `src-tauri/tests/mcp_services_tests.rs`

---

## 2. Tools Reference

### Projects
| Tool | Purpose | Key Arguments | Notes |
| :--- | :--- | :--- | :--- |
| `list_projects` | Lists all projects with time totals and task counts | `include_system` (bool, default `false`) | Returns `id`, `name`, `color`, `total_time_seconds`, `formatted_total_time`, `active_task_count`, `completed_task_count`. |

### Tasks & Deadlines
| Tool | Purpose | Key Arguments | Notes |
| :--- | :--- | :--- | :--- |
| `create_task` | Creates a new task | `title` (required), `project_name` or `project_id`, `section_name` or `section_id`, `deadline`, `description` | Resolves `project_name` case-insensitively. |
| `set_task_deadline` | Updates or clears deadline | `task_id` (int), `deadline` (string or `null`) | Auto-syncs with Google Calendar. |
| `find_tasks` | Searches task titles/descriptions | `query` (string), `include_completed` (bool), `limit` (int) | Best for finding task IDs before mutation. |
| `list_due_tasks` | Lists tasks in a date range | `start_date`, `end_date` | Dates formatted as `YYYY-MM-DD` or `YYYY-MM-DDTHH:mm`. |
| `get_task` | Fetches task by exact ID | `task_id` (int) | Returns full `Task` model with `total_time_seconds`. |
| `set_task_completed` | Marks complete/incomplete | `task_id` (int), `completed` (bool) | Syncs completion status to Google Calendar. |

### Hours & Time Tracking
| Tool | Purpose | Key Arguments | Notes |
| :--- | :--- | :--- | :--- |
| `get_time_stats` | Today's tasks, week totals, project hours | `include_active_timer` (bool, default `true`) | Returns formatted and raw second totals. |
| `log_time` | Manually logs time on a task | `task_id` (int), `duration_minutes` or `duration_seconds`, `note` (optional) | Updates task and project denormalized time totals. |
| `get_timer_status` | Checks active timer status | (None) | Returns `is_running`, `task_id`, `task_title`, `elapsed_seconds`, `formatted_elapsed_time`. |
| `start_timer` | Starts timing a task | `task_id` (int) | Begins active timer session. |
| `stop_timer` | Stops active timer | (None) | Generates `time_entries` record and updates parent totals. |

---

## 3. Best Practices & Workflows

### Lookup Before Mutation
When the user refers to a task by title or natural description:
1. Call `find_tasks` first to retrieve candidate tasks and their exact `id`.
2. Confirm the match before calling `set_task_deadline`, `set_task_completed`, `log_time`, or `start_timer`.

### Creating Tasks in Named Projects
Instead of hardcoding numerical IDs, prefer using `project_name`:
```json
{
  "title": "Draft quarterly review",
  "project_name": "Work",
  "deadline": "2026-06-15"
}
```

### Logging Time & Hours
You can provide either `duration_minutes` or `duration_seconds`:
```json
{
  "task_id": 42,
  "duration_minutes": 45,
  "note": "Refactored auth endpoints"
}
```

---

## 4. Architectural Rules for Development

When modifying or adding tools to `todoz-mcp`:

1. **Shared Services Only**: Never execute direct raw SQL queries in `todoz-mcp.rs`. All database reads and mutations must go through the shared Rust services in `src-tauri/src/services/`:
   - `services::projects_service`: Project listing, search, and formatting.
   - `services::tasks_service`: Task CRUD, deadline modification, and lookup.
   - `services::time_service`: Manual time entries, time stats calculation.
   - `services::timer_service`: Live timer lifecycle and staleness recovery.
2. **Denormalization Integrity**: Ensure any duration change updates parent tasks and projects via `apply_task_and_parent_time_delta`.
3. **Google Calendar Consistency**: Always trigger `sync_task_to_calendar` or `delete_from_calendar` when task deadlines or completion states change.

---

## 5. Build, Test, and Verify

Run all commands from `src-tauri`:

```powershell
# Development run
cargo run --bin todoz-mcp

# Build debug binary
cargo build --bin todoz-mcp

# Build release binary
cargo build --release --bin todoz-mcp

# Run integration tests
cargo test --test mcp_services_tests

# Clippy linter
cargo clippy --bin todoz-mcp -- -D warnings
```
