---
name: stats-integration
description: Integrating non-task events (e.g., breaks) into standard project time stats
---

# Stats Integration for System Events

This skill covers how to integrate non-standard activities, such as breaks or meetings, into an existing task-based time tracking system so they appear in analytics and pie charts.

## 1. Dedicated System Project Pattern
Instead of adding new database columns or logic to the stats engine, create specialized "System Projects" and "System Tasks" to hold the time entries for these events.

### Implementation
When a non-task event occurs (e.g., a break finishes):
1. **Find or Create Project**: Check for a project named "Breaks".
2. **Find or Create Task**: Check for a task named "Break" under that project.
3. **Log Entry**: Insert a `manual` type `time_entry` for that task.
4. **Update Aggregates**: Trigger a delta update on the project's `total_time_seconds` so the pie chart reflects it immediately.

```rust
// Backend Logic (Rust)
pub fn log_break_time(db: State<DbConnection>, duration: i64) -> Result<()> {
    let conn = db.lock();
    // 1. Get Project ID (Create if missing)
    // 2. Get Task ID (Create if missing)
    // 3. Insert into time_entries
    // 4. apply_task_and_parent_time_delta(...)
}
```

## 2. Benefits
- **Zero Schema Changes**: Reuses existing `time_entries`, `projects`, and `tasks` tables.
- **Instant Visualization**: Automatically picked up by SQL queries that group by project or task.
- **Flexibility**: Can be extended to "Meetings", "Commute", etc., by just changing the names.
