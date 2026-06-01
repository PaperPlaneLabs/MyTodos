use my_todos_lib::db::{initialize_connection, initialize_schema, DbConnection, Task};
use my_todos_lib::google;
use my_todos_lib::services::tasks_service;
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
        eprintln!("mytodos-mcp failed: {}", error);
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
                "name": "mytodos-mcp",
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
            "name": "create_task",
            "description": "Create a MyTodos task. Deadlines must be normalized as YYYY-MM-DD or YYYY-MM-DDTHH:mm.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "description": { "type": ["string", "null"] },
                    "project_id": { "type": ["integer", "null"] },
                    "section_id": { "type": ["integer", "null"] },
                    "deadline": { "type": ["string", "null"], "description": "YYYY-MM-DD or YYYY-MM-DDTHH:mm" }
                },
                "required": ["title"]
            }
        },
        {
            "name": "set_task_deadline",
            "description": "Set or clear the deadline for an existing MyTodos task by exact task id.",
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
            "description": "Find MyTodos tasks by title or description text before making a change.",
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
            "description": "List MyTodos tasks whose deadlines fall within the given inclusive lexical date range.",
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
            "description": "Get one MyTodos task by exact task id.",
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
            "description": "Mark a MyTodos task complete or incomplete by exact task id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": { "type": "integer" },
                    "completed": { "type": "boolean" }
                },
                "required": ["task_id", "completed"]
            }
        }
    ])
}

fn call_tool(state: &McpState, name: &str, arguments: Value) -> std::result::Result<Value, String> {
    let output = match name {
        "create_task" => create_task_tool(state, arguments),
        "set_task_deadline" => set_task_deadline_tool(state, arguments),
        "find_tasks" => find_tasks_tool(state, arguments),
        "list_due_tasks" => list_due_tasks_tool(state, arguments),
        "get_task" => get_task_tool(state, arguments),
        "set_task_completed" => set_task_completed_tool(state, arguments),
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

fn create_task_tool(state: &McpState, arguments: Value) -> std::result::Result<Value, String> {
    let title = required_string(&arguments, "title")?;
    let description = optional_string(&arguments, "description")?;
    let project_id = optional_i64(&arguments, "project_id")?;
    let section_id = optional_i64(&arguments, "section_id")?;
    let deadline = normalize_optional_deadline(optional_string(&arguments, "deadline")?)?;

    let task = {
        let conn = state.db.lock();
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
