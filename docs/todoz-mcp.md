# Todoz MCP Server

Todoz includes a local stdio MCP server for trusted AI clients that need to read or update tasks.

## Binary

Run from `src-tauri`:

```powershell
cargo run --bin todoz-mcp
```

For an installed build, point the MCP client at the compiled `todoz-mcp` executable.

For local client configuration, build the executable first:

```powershell
cd F:\personal_projects\MyTodos\src-tauri
cargo build --bin todoz-mcp
```

The debug executable will be:

```text
F:\personal_projects\MyTodos\src-tauri\target\debug\todoz-mcp.exe
```

For regular personal use, prefer a release build:

```powershell
cd F:\personal_projects\MyTodos\src-tauri
cargo build --release --bin todoz-mcp
```

The release executable will be:

```text
F:\personal_projects\MyTodos\src-tauri\target\release\todoz-mcp.exe
```

## Tools

### Projects
- `list_projects`: lists all Todoz projects with ID, name, description, color, total time tracked (seconds and formatted duration string), and active/completed task counts. Supports optional `include_system` boolean.

### Tasks & Deadlines
- `create_task`: creates a task. Supports `title` (required), optional `description`, `project_id` OR `project_name` (case-insensitive lookup), optional `section_id` OR `section_name`, and `deadline` (`YYYY-MM-DD` or `YYYY-MM-DDTHH:mm`).
- `set_task_deadline`: sets or clears (`null`) a deadline by exact task id. Automatically syncs to Google Calendar when connected.
- `find_tasks`: searches task title and description text.
- `list_due_tasks`: lists tasks whose deadlines fall within an inclusive lexical date range.
- `get_task`: returns a task by exact id.
- `set_task_completed`: marks a task complete or incomplete by exact id.

### Hours & Time Tracking
- `get_time_stats`: returns today's tasks and time spent, this week's daily breakdown, and total hours/seconds per project. Supports optional `include_active_timer` boolean (default `true`).
- `log_time`: manually logs time worked on a task (`task_id` required, `duration_minutes` or `duration_seconds`, optional `note`). Increments task and project denormalized time totals.
- `get_timer_status`: returns active timer status (whether running, timed task id and title, elapsed time).
- `start_timer`: starts the active timer for a specific task by task id.
- `stop_timer`: stops the running active timer and saves the elapsed duration as a time entry.

Deadlines must be normalized before tool calls as `YYYY-MM-DD` or `YYYY-MM-DDTHH:mm`.

## Data Path

The MCP server uses the same SQLite database as the Tauri app. Task writes go through shared Rust services rather than a separate raw-SQL path. Deadline changes are synced to Google Calendar when the app is connected to Google Calendar.

## Safety Model

The first version requires exact task ids for mutating existing tasks. Clients should call `find_tasks` first when the user refers to a task by title or natural language.

## Gemini CLI Configuration

Gemini CLI reads MCP servers from `settings.json` under `mcpServers`. Use a user-level config for personal use:

```text
%USERPROFILE%\.gemini\settings.json
```

Example:

```json
{
  "mcpServers": {
    "todoz": {
      "command": "F:\\personal_projects\\MyTodos\\src-tauri\\target\\release\\todoz-mcp.exe",
      "timeout": 30000,
      "trust": false
    }
  }
}
```

Development-only example using Cargo:

```json
{
  "mcpServers": {
    "todoz": {
      "command": "cargo",
      "args": ["run", "--quiet", "--bin", "todoz-mcp"],
      "cwd": "F:\\personal_projects\\MyTodos\\src-tauri",
      "timeout": 30000,
      "trust": false
    }
  }
}
```

Keep `trust: false` at first so Gemini asks before tool execution. After changing settings, restart Gemini CLI and run `/mcp` to inspect the connected server and tools.

## Claude Code Configuration

For Claude Code, add a project-scoped `.mcp.json` when you want the server enabled only for this repo:

```json
{
  "mcpServers": {
    "todoz": {
      "type": "stdio",
      "command": "F:\\personal_projects\\MyTodos\\src-tauri\\target\\release\\todoz-mcp.exe",
      "args": [],
      "env": {}
    }
  }
}
```

Or add it from the CLI:

```powershell
claude mcp add --transport stdio todoz -- F:\personal_projects\MyTodos\src-tauri\target\release\todoz-mcp.exe
```

Run `claude mcp list` or `/mcp` inside Claude Code to verify the tool list.

## Claude Desktop Configuration

Claude Desktop can use a local stdio server via `claude_desktop_config.json`. On Windows, the config is typically under:

```text
%APPDATA%\Claude\claude_desktop_config.json
```

Example:

```json
{
  "mcpServers": {
    "todoz": {
      "type": "stdio",
      "command": "F:\\personal_projects\\MyTodos\\src-tauri\\target\\release\\todoz-mcp.exe",
      "args": [],
      "env": {}
    }
  }
}
```

Restart Claude Desktop after editing the file. If the server does not appear, confirm the executable exists and that Claude Desktop has permission to run local MCP servers.

## Example Prompts

Use prompts that encourage lookup before mutation:

```text
List all my Todoz projects and how much time has been tracked on each.
```

```text
Create a Todoz task titled "Submit invoice" in the "Work" project with deadline 2026-06-05.
```

```text
Log 45 minutes on task #12 with the note "Initial draft and outline".
```

```text
How many hours have I worked today and this week according to Todoz?
```

```text
Check if my timer is currently running, and if so, what task is it tracking?
```

```text
Use Todoz to find tasks matching "electricity bill". If there is one clear match, set its deadline to 2026-06-03T18:00.
```

```text
List Todoz tasks due between 2026-06-01 and 2026-06-07.
```
