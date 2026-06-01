# MyTodos MCP Server

MyTodos includes a local stdio MCP server for trusted AI clients that need to read or update tasks.

## Binary

Run from `src-tauri`:

```powershell
cargo run --bin mytodos-mcp
```

For an installed build, point the MCP client at the compiled `mytodos-mcp` executable.

## Tools

- `create_task`: creates a task with optional description, project, section, and deadline.
- `set_task_deadline`: sets or clears a deadline by exact task id.
- `find_tasks`: searches task title and description text.
- `list_due_tasks`: lists tasks in a deadline range.
- `get_task`: returns a task by exact id.
- `set_task_completed`: marks a task complete or incomplete by exact id.

Deadlines must be normalized before tool calls as `YYYY-MM-DD` or `YYYY-MM-DDTHH:mm`.

## Data Path

The MCP server uses the same SQLite database as the Tauri app. Task writes go through shared Rust services rather than a separate raw-SQL path. Deadline changes are synced to Google Calendar when the app is connected to Google Calendar.

## Safety Model

The first version requires exact task ids for mutating existing tasks. Clients should call `find_tasks` first when the user refers to a task by title or natural language.
