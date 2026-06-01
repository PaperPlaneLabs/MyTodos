# MyTodos MCP Server

MyTodos includes a local stdio MCP server for trusted AI clients that need to read or update tasks.

## Binary

Run from `src-tauri`:

```powershell
cargo run --bin mytodos-mcp
```

For an installed build, point the MCP client at the compiled `mytodos-mcp` executable.

For local client configuration, build the executable first:

```powershell
cd F:\personal_projects\MyTodos\src-tauri
cargo build --bin mytodos-mcp
```

The debug executable will be:

```text
F:\personal_projects\MyTodos\src-tauri\target\debug\mytodos-mcp.exe
```

For regular personal use, prefer a release build:

```powershell
cd F:\personal_projects\MyTodos\src-tauri
cargo build --release --bin mytodos-mcp
```

The release executable will be:

```text
F:\personal_projects\MyTodos\src-tauri\target\release\mytodos-mcp.exe
```

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

## Gemini CLI Configuration

Gemini CLI reads MCP servers from `settings.json` under `mcpServers`. Use a user-level config for personal use:

```text
%USERPROFILE%\.gemini\settings.json
```

Example:

```json
{
  "mcpServers": {
    "mytodos": {
      "command": "F:\\personal_projects\\MyTodos\\src-tauri\\target\\release\\mytodos-mcp.exe",
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
    "mytodos": {
      "command": "cargo",
      "args": ["run", "--quiet", "--bin", "mytodos-mcp"],
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
    "mytodos": {
      "type": "stdio",
      "command": "F:\\personal_projects\\MyTodos\\src-tauri\\target\\release\\mytodos-mcp.exe",
      "args": [],
      "env": {}
    }
  }
}
```

Or add it from the CLI:

```powershell
claude mcp add --transport stdio mytodos -- F:\personal_projects\MyTodos\src-tauri\target\release\mytodos-mcp.exe
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
    "mytodos": {
      "type": "stdio",
      "command": "F:\\personal_projects\\MyTodos\\src-tauri\\target\\release\\mytodos-mcp.exe",
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
Use MyTodos to find tasks matching "electricity bill". If there is one clear match, set its deadline to 2026-06-03T18:00.
```

```text
Create a MyTodos task titled "Submit invoice" with deadline 2026-06-05.
```

```text
List MyTodos tasks due between 2026-06-01 and 2026-06-07.
```
