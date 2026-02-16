<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";

  let selectedDateStr = $derived(
    calendarStore.selectedDate
      ? calendarStore.dateToString(calendarStore.selectedDate)
      : null,
  );
  let tasks = $derived(
    selectedDateStr ? calendarStore.getTasksForDate(selectedDateStr) : [],
  );
  let events = $derived(
    selectedDateStr ? calendarStore.getEventsForDate(selectedDateStr) : [],
  );

  function getProjectColor(projectId: number | null): string {
    if (!projectId) return "var(--text-tertiary)";
    const project = projectStore.projects.find((p) => p.id === projectId);
    return project?.color || "var(--text-tertiary)";
  }

  function handleTaskClick(task: any) {
    uiStore.openTaskModal({ taskId: task.id });
  }

  function handleAddTask() {
    if (selectedDateStr) {
      uiStore.openTaskModal({ deadline: selectedDateStr });
    }
  }

  function formatTime(deadline: string | null): string | null {
    if (!deadline || !deadline.includes("T")) return null;
    const timePart = deadline.split("T")[1];
    if (!timePart) return null;
    return timePart.substring(0, 5);
  }
</script>

<div class="day-task-list">
  <div class="list-header">
    <h3>
      {#if calendarStore.selectedDate}
        {calendarStore.selectedDate.toLocaleDateString("en-US", {
          weekday: "long",
          month: "long",
          day: "numeric",
        })}
      {:else}
        Select a date
      {/if}
    </h3>
    {#if calendarStore.selectedDate}
      <button class="btn-add" onclick={handleAddTask}>
        <span>+ Add Task</span>
      </button>
    {/if}
  </div>

  <div class="list-content">
    {#if !calendarStore.selectedDate}
      <div class="empty-state">
        <p>Select a date from the calendar to see tasks</p>
      </div>
    {:else if tasks.length === 0 && events.length === 0}
      <div class="empty-state">
        <p>No tasks or events for this day</p>
      </div>
    {:else}
      {#if events.length > 0}
        <div class="section">
          <h4>Events</h4>
          {#each events as event}
            <div
              class="event-item"
              style="border-left-color: {event.color || 'var(--accent)'}"
            >
              <span class="event-title">{event.title}</span>
            </div>
          {/each}
        </div>
      {/if}

      {#if tasks.length > 0}
        <div class="section">
          <h4>Tasks</h4>
          {#each tasks as task}
            <button
              class="task-item"
              onclick={() => handleTaskClick(task)}
              type="button"
            >
              <div
                class="project-indicator"
                style="background-color: {getProjectColor(
                  task.project_id ?? null,
                )}"
              ></div>
              <div class="task-info">
                <div class="task-main-row">
                  <span class="task-title" class:completed={task.completed}
                    >{task.title}</span
                  >
                  {#if formatTime(task.deadline ?? null)}
                    <span class="task-scheduled-time"
                      >{formatTime(task.deadline ?? null)}</span
                    >
                  {/if}
                </div>
                <div class="task-meta">
                  {#if task.total_time_seconds > 0}
                    <span class="task-time">
                      <TimeDisplay
                        seconds={task.total_time_seconds}
                        format="short"
                      />
                    </span>
                  {/if}
                </div>
              </div>
              {#if task.completed}
                <span class="completed-badge">✓</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .day-task-list {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .list-header {
    padding: var(--spacing-md);
    background: var(--bg-hover);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .list-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .btn-add {
    background: var(--accent);
    color: white;
    border: none;
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .btn-add:hover {
    background: var(--accent-dark);
  }

  .list-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .section h4 {
    margin: 0 0 var(--spacing-xs) 0;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .event-item {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-primary);
    border-left: 3px solid var(--accent);
    border-radius: var(--radius-sm);
    font-size: 14px;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    border: 1px solid transparent;
  }

  .task-item:hover {
    background: var(--bg-hover);
    border-color: var(--border);
  }

  .project-indicator {
    width: 4px;
    height: 24px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .task-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .task-main-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
  }

  .task-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .task-scheduled-time {
    font-size: 11px;
    font-weight: 600;
    color: var(--accent);
    background: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    flex-shrink: 0;
  }

  .task-title.completed {
    color: var(--text-tertiary);
    text-decoration: line-through;
  }

  .task-meta {
    display: flex;
    gap: var(--spacing-sm);
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .completed-badge {
    color: var(--success);
    font-weight: bold;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
    text-align: center;
    font-size: 14px;
  }
</style>
