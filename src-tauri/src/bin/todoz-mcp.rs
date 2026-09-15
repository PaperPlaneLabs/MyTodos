use my_todos_lib::db::{initialize_connection, initialize_schema, DbConnection, Task};
use my_todos_lib::google;
use my_todos_lib::services::{projects_service, tasks_service, time_service, timer_service};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

struct McpState {
    db: DbConnection,
    google_state: google::GoogleCalendarState,
    runtime: tokio::runtime::Runtime,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("todoz-mcp failed: {}", error);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let db = initialize_connection()?;
    {
        let conn = db.lock();
        initialize_schema(&conn)?;
    }

    let state = McpState {
        db,
        google_state: google::create_google_state(),
        runtime: tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?,
    };

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(error) => {
                write_response(
                    &mut stdout,
                    json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": { "code": -32700, "message": format!("Parse error: {}", error) }
                    }),
                )?;
                continue;
            }
        };

        if request.id.is_none() {
            continue;
        }

        let id = request.id.clone().unwrap_or(Value::Null);
        let response = match handle_request(&state, &request) {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(message) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": message } })
            }
        };

        write_response(&mut stdout, response)?;
    }

    Ok(())
}

fn write_response(stdout: &mut io::Stdout, response: Value) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *stdout, &response)?;
    writeln!(stdout)?;
    stdout.flush()?;
    Ok(())
}

fn handle_request(
    state: &McpState,
    request: &JsonRpcRequest,
) -> std::result::Result<Value, String> {
    match request.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "todoz-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_definitions() })),
        "tools/call" => {
            let params: ToolCallParams = serde_json::from_value(request.params.clone())
                .map_err(|error| format!("Invalid tools/call params: {}", error))?;
            call_tool(state, &params.name, params.arguments)
        }
        method => Err(format!("Unsupported method: {}", method)),
    }
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_projects",
            "description": "List all Todoz projects with their id, name, description, color, total tracked time/hours, and active/completed task counts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "include_system": { "type": "boolean", "default": false, "description": "Whether to include system projects (e.g. Away, Breaks)" }
                }
            }
        },
        {
            "name": "create_task",
            "description": "Create a Todoz task. Supports assigning to a project by project_id or project_name, optional section_id or section_name, and setting a deadline (YYYY-MM-DD or YYYY-MM-DDTHH:mm).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Task title" },
                    "description": { "type": ["string", "null"], "description": "Task description or notes" },
                    "project_id": { "type": ["integer", "null"], "description": "Numeric ID of the project" },
                    "project_name": { "type": ["string", "null"], "description": "Name of the project (case-insensitive lookup used if project_id is omitted)" },
                    "section_id": { "type": ["integer", "null"], "description": "Numeric ID of the section within the project" },
                    "section_name": { "type": ["string", "null"], "description": "Name of the section within the project (case-insensitive lookup used if section_id is omitted)" },
                    "deadline": { "type": ["string", "null"], "description": "YYYY-MM-DD or YYYY-MM-DDTHH:mm" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "set_task_deadline",
            "description": "Set or clear the deadline for an existing Todoz task by exact task id. Automatically syncs to Google Calendar if connected.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer" },
                    "deadline": { "type": ["string", "null"], "description": "YYYY-MM-DD or YYYY-MM-DDTHH:mm; null clears the deadline" }
                },
                "required": ["task_id", "deadline"]
            }
        },
        {
            "name": "find_tasks",
            "description": "Find Todoz tasks by title or description text before making a change.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "include_completed": { "type": "boolean", "default": false },
                    "limit": { "type": "integer", "default": 10, "minimum": 1, "maximum": 50 }
                },
                "required": ["query"]
            }
        },
        {
            "name": "list_due_tasks",
            "description": "List Todoz tasks whose deadlines fall within the given inclusive lexical date range.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "start_date": { "type": "string", "description": "YYYY-MM-DD or YYYY-MM-DDTHH:mm" },
                    "end_date": { "type": "string", "description": "YYYY-MM-DD or YYYY-MM-DDTHH:mm" }
                },
                "required": ["start_date", "end_date"]
            }
        },
        {
            "name": "get_task",
            "description": "Get one Todoz task by exact task id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer" }
                },
                "required": ["task_id"]
            }
        },
        {
            "name": "set_task_completed",
            "description": "Mark a Todoz task complete or incomplete by exact task id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer" },
                    "completed": { "type": "boolean" }
                },
                "required": ["task_id", "completed"]
            }
        },
        {
            "name": "get_time_stats",
            "description": "Get time tracking statistics including today's task hours, this week's daily breakdown, and total hours per project.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "include_active_timer": { "type": "boolean", "default": true, "description": "Whether to include running active timer duration in stats" }
                }
            }
        },
        {
            "name": "log_time",
            "description": "Manually log time worked on a task (creates a manual time entry and increments task and project totals). Provide duration_minutes or duration_seconds.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer", "description": "Numeric ID of the task" },
                    "duration_minutes": { "type": ["integer", "null"], "description": "Duration worked in minutes" },
                    "duration_seconds": { "type": ["integer", "null"], "description": "Duration worked in seconds" },
                    "note": { "type": ["string", "null"], "description": "Optional note describing the work done" }
                },
                "required": ["task_id"]
            }
        },
        {
            "name": "get_timer_status",
            "description": "Get the current active timer status (is running, current task id, task title, elapsed time, started time).",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "start_timer",
            "description": "Start the timer for a specific task by task id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer", "description": "Numeric ID of the task to time" }
                },
                "required": ["task_id"]
            }
        },
        {
            "name": "stop_timer",
            "description": "Stop the current active timer and record the time entry for the tracked task.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ])
}

fn call_tool(state: &McpState, name: &str, arguments: Value) -> std::result::Result<Value, String> {
    let output = match name {
        "list_projects" => list_projects_tool(state, arguments),
        "create_task" => create_task_tool(state, arguments),
        "set_task_deadline" => set_task_deadline_tool(state, arguments),
        "find_tasks" => find_tasks_tool(state, arguments),
        "list_due_tasks" => list_due_tasks_tool(state, arguments),
        "get_task" => get_task_tool(state, arguments),
        "set_task_completed" => set_task_completed_tool(state, arguments),
        "get_time_stats" => get_time_stats_tool(state, arguments),
        "log_time" => log_time_tool(state, arguments),
        "get_timer_status" => get_timer_status_tool(state, arguments),
        "start_timer" => start_timer_tool(state, arguments),
        "stop_timer" => stop_timer_tool(state, arguments),
        _ => Err(format!("Unknown tool: {}", name)),
    }?;

    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
            }
        ]
    }))
}

fn list_projects_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let include_system = arguments
        .get("include_system")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let projects = {
        let conn = state.db.lock();
        projects_service::list_projects(&conn, include_system)
            .map_err(|error| error.to_string())?
    };

    let count = projects.len();
    Ok(json!({
        "projects": projects,
        "count": count,
        "confirmation": {
            "required": false
        }
    }))
}

fn create_task_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let title = required_string(&arguments, "title")?;
    let description = optional_string(&arguments, "description")?;
    let mut project_id = optional_i64(&arguments, "project_id")?;
    let project_name = optional_string(&arguments, "project_name")?;
    let mut section_id = optional_i64(&arguments, "section_id")?;
    let section_name = optional_string(&arguments, "section_name")?;
    let deadline = normalize_optional_deadline(optional_string(&arguments, "deadline")?)?;

    let task = {
        let conn = state.db.lock();

        if project_id.is_none() {
            if let Some(ref pname) = project_name {
                let project = projects_service::find_project_by_name(&conn, pname)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| format!("Project '{}' not found", pname))?;
                project_id = Some(project.id);
            }
        }

        if section_id.is_none() {
            if let Some(ref sname) = section_name {
                let pid = project_id.ok_or_else(|| {
                    "Cannot specify section_name without project_id or project_name".to_string()
                })?;
                let section = projects_service::find_section_by_name(&conn, pid, sname)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| format!("Section '{}' not found in project", sname))?;
                section_id = Some(section.id);
            }
        }

        tasks_service::create_task(&conn, project_id, section_id, title, description, deadline)
            .map_err(|error| error.to_string())?
    };

    sync_task_to_calendar(state, task.id)?;
    Ok(task_result(task))
}

fn set_task_deadline_tool(
    state: &McpState,
    arguments: Value,
) -> std::result::Result<Value, String> {
    let task_id = required_i64(&arguments, "task_id")?;
    let deadline = normalize_optional_deadline(optional_string(&arguments, "deadline")?)?;

    let task = {
        let conn = state.db.lock();
        tasks_service::set_task_deadline(&conn, task_id, deadline)
            .map_err(|error| error.to_string())?
    };

    sync_task_to_calendar(state, task.id)?;
    Ok(task_result(task))
}

fn find_tasks_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let query = required_string(&arguments, "query")?;
    let include_completed = arguments
        .get("include_completed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let limit = optional_i64(&arguments, "limit")?.unwrap_or(10);

    let tasks = {
        let conn = state.db.lock();
        tasks_service::find_tasks(&conn, &query, include_completed, limit)
            .map_err(|error| error.to_string())?
    };

    Ok(tasks_result(tasks))
}

fn list_due_tasks_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let start_date = normalize_deadline(required_string(&arguments, "start_date")?)?;
    let end_date = normalize_deadline(required_string(&arguments, "end_date")?)?;

    let tasks = {
        let conn = state.db.lock();
        tasks_service::list_due_tasks(&conn, &start_date, &end_date)
            .map_err(|error| error.to_string())?
    };

    Ok(tasks_result(tasks))
}

fn get_task_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let task_id = required_i64(&arguments, "task_id")?;
    let task = {
        let conn = state.db.lock();
        tasks_service::get_task(&conn, task_id).map_err(|error| error.to_string())?
    };

    Ok(task_result(task))
}

fn set_task_completed_tool(
    state: &McpState,
    arguments: Value,
) -> std::result::Result<Value, String> {
    let task_id = required_i64(&arguments, "task_id")?;
    let completed = arguments
        .get("completed")
        .and_then(Value::as_bool)
        .ok_or_else(|| "Missing required boolean argument: completed".to_string())?;

    let (task, previous_google_event_id) = {
        let conn = state.db.lock();
        let previous =
            tasks_service::get_task(&conn, task_id).map_err(|error| error.to_string())?;
        let task = tasks_service::set_task_completed(&conn, task_id, completed)
            .map_err(|error| error.to_string())?;
        (task, previous.google_event_id)
    };

    if completed {
        if let Some(event_id) = previous_google_event_id {
            delete_from_calendar(state, &event_id)?;
        }
    } else {
        sync_task_to_calendar(state, task.id)?;
    }

    Ok(task_result(task))
}

fn get_time_stats_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let include_active_timer = arguments
        .get("include_active_timer")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let stats = {
        let conn = state.db.lock();
        time_service::get_time_stats(&conn, include_active_timer)
            .map_err(|error| error.to_string())?
    };

    let total_today_seconds: i64 = stats.today_tasks.iter().map(|t| t.total_seconds).sum();
    let total_week_seconds: i64 = stats.week_daily.iter().map(|d| d.total_seconds).sum();

    Ok(json!({
        "time_stats": stats,
        "summary": {
            "today_total_seconds": total_today_seconds,
            "today_total_formatted": time_service::format_duration(total_today_seconds),
            "week_total_seconds": total_week_seconds,
            "week_total_formatted": time_service::format_duration(total_week_seconds)
        },
        "confirmation": {
            "required": false
        }
    }))
}

fn log_time_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let task_id = required_i64(&arguments, "task_id")?;
    let duration_seconds = optional_i64(&arguments, "duration_seconds")?;
    let duration_minutes = optional_i64(&arguments, "duration_minutes")?;
    let note = optional_string(&arguments, "note")?;

    let seconds = match (duration_seconds, duration_minutes) {
        (Some(s), _) if s > 0 => s,
        (_, Some(m)) if m > 0 => m * 60,
        (Some(_), _) => return Err("duration_seconds must be positive".to_string()),
        (_, Some(_)) => return Err("duration_minutes must be positive".to_string()),
        (None, None) => {
            return Err("Must provide either duration_minutes or duration_seconds".to_string())
        }
    };

    let (entry, updated_task) = {
        let conn = state.db.lock();
        let entry = time_service::create_manual_entry(&conn, task_id, seconds, note)
            .map_err(|error| error.to_string())?;
        let updated_task = tasks_service::get_task(&conn, task_id)
            .map_err(|error| error.to_string())?;
        (entry, updated_task)
    };

    Ok(json!({
        "time_entry": entry,
        "task": updated_task,
        "formatted_duration": time_service::format_duration(seconds),
        "confirmation": {
            "required": false
        }
    }))
}

fn get_timer_status_tool(
    state: &McpState,
    _arguments: Value,
) -> std::result::Result<Value, String> {
    timer_service::recover_stale_active_timer(&state.db).map_err(|error| error.to_string())?;
    let timer = timer_service::get_active_timer(&state.db).map_err(|error| error.to_string())?;

    let is_running = timer.as_ref().map(|t| t.is_running).unwrap_or(false);
    let elapsed_formatted = timer
        .as_ref()
        .map(|t| time_service::format_duration(t.elapsed_seconds));

    Ok(json!({
        "active_timer": timer,
        "is_running": is_running,
        "formatted_elapsed_time": elapsed_formatted,
        "confirmation": {
            "required": false
        }
    }))
}

fn start_timer_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let task_id = required_i64(&arguments, "task_id")?;
    let timer = timer_service::start_timer(&state.db, task_id)
        .map_err(|error| error.to_string())?;

    Ok(json!({
        "active_timer": timer,
        "message": format!("Timer started for task {}", task_id),
        "confirmation": {
            "required": false
        }
    }))
}

fn stop_timer_tool(state: &McpState, _arguments: Value) -> std::result::Result<Value, String> {
    let entry = timer_service::stop_timer(&state.db)
        .map_err(|error| error.to_string())?;

    let formatted_duration = time_service::format_duration(entry.duration_seconds);
    Ok(json!({
        "time_entry": entry,
        "formatted_duration": formatted_duration,
        "message": format!("Timer stopped. Recorded {}.", formatted_duration),
        "confirmation": {
            "required": false
        }
    }))
}

fn sync_task_to_calendar(state: &McpState, task_id: i64) -> std::result::Result<(), String> {
    state
        .runtime
        .block_on(google::sync::sync_task_to_calendar(
            state.db.clone(),
            &state.google_state,
            task_id,
        ))
        .map_err(|error| format!("Task saved, but Google Calendar sync failed: {}", error))
}

fn delete_from_calendar(
    state: &McpState,
    google_event_id: &str,
) -> std::result::Result<(), String> {
    state
        .runtime
        .block_on(google::sync::delete_from_calendar(
            &state.google_state,
            google_event_id,
        ))
        .map_err(|error| format!("Task saved, but Google Calendar delete failed: {}", error))
}

fn task_result(task: Task) -> Value {
    json!({
        "task": task,
        "confirmation": {
            "required": false
        }
    })
}

fn tasks_result(tasks: Vec<Task>) -> Value {
    let count = tasks.len();
    json!({
        "tasks": tasks,
        "count": count,
        "confirmation": {
            "required": false
        }
    })
}

fn required_string(arguments: &Value, key: &str) -> std::result::Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("Missing required string argument: {}", key))
}

fn optional_string(arguments: &Value, key: &str) -> std::result::Result<Option<String>, String> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or_else(|| format!("Expected string or null for argument: {}", key)),
    }
}

fn required_i64(arguments: &Value, key: &str) -> std::result::Result<i64, String> {
    arguments
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("Missing required integer argument: {}", key))
}

fn optional_i64(arguments: &Value, key: &str) -> std::result::Result<Option<i64>, String> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| format!("Expected integer or null for argument: {}", key)),
    }
}

fn normalize_optional_deadline(
    deadline: Option<String>,
) -> std::result::Result<Option<String>, String> {
    deadline.map(normalize_deadline).transpose()
}

fn normalize_deadline(deadline: String) -> std::result::Result<String, String> {
    let deadline = deadline.trim();
    if chrono::NaiveDate::parse_from_str(deadline, "%Y-%m-%d").is_ok()
        || chrono::NaiveDateTime::parse_from_str(deadline, "%Y-%m-%dT%H:%M").is_ok()
        || chrono::NaiveDateTime::parse_from_str(deadline, "%Y-%m-%dT%H:%M:%S").is_ok()
    {
        Ok(deadline.to_string())
    } else {
        Err(format!(
            "Invalid deadline '{}'. Use YYYY-MM-DD or YYYY-MM-DDTHH:mm.",
            deadline
        ))
    }
}
