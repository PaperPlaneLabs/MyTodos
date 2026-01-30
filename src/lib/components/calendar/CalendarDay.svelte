<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import CalendarHeader from './CalendarHeader.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  
  let selectedDate = $derived(calendarStore.selectedDate || calendarStore.currentDate);
  let dateStr = $derived(calendarStore.dateToString(selectedDate));
  let tasks = $derived(calendarStore.getTasksForDate(dateStr));
  let events = $derived(calendarStore.getEventsForDate(dateStr));
  
  function formatDate(date: Date): string {
    return date.toLocaleDateString('en-US', { 
      weekday: 'long', 
      year: 'numeric', 
      month: 'long', 
      day: 'numeric' 
    });
  }
  
  function handleTaskClick(task: any) {
    uiStore.openTaskModal({ task, deadline: dateStr });
  }
</script>

<CalendarHeader />

<div class="day-view">
  <div class="day-header">
    <h2 class="day-title">{formatDate(selectedDate)}</h2>
    <button 
      class="add-task-btn"
      onclick={() => uiStore.openTaskModal({ deadline: dateStr })}
    >
      + Add Task
    </button>
  </div>
  
  <div class="day-content">
    {#if tasks.length === 0 && events.length === 0}
      <div class="empty-state">
        <p>No tasks or events scheduled for this day</p>
        <button onclick={() => uiStore.openTaskModal({ deadline: dateStr })}>
          Add a task
        </button>
      </div>
    {:else}
      <div class="events-section">
        {#each events as event}
          <div 
            class="event-card"
            style="border-left-color: {event.color || 'var(--accent)'}"
          >
            <div class="event-time">
              {event.is_all_day ? 'All day' : ''}
            </div>
            <div class="event-title">{event.title}</div>
            {#if event.description}
              <div class="event-description">{event.description}</div>
            {/if}
          </div>
        {/each}
      </div>
      
      <div class="tasks-section">
        <h3>Tasks</h3>
        {#each tasks as task}
          <div 
            class="task-card"
            class:completed={task.completed}
            onclick={() => handleTaskClick(task)}
            role="button"
            tabindex="0"
          >
            <div class="task-checkbox">
              <input 
                type="checkbox" 
                checked={task.completed}
                readonly
              />
            </div>
            <div class="task-info">
              <div class="task-title">{task.title}</div>
              {#if task.project_id}
                <div class="task-project">Project ID: {task.project_id}</div>
              {/if}
            </div>
            <div class="task-actions">
              <span class="edit-hint">✏️</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .day-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  
  .day-header {
    padding: var(--spacing-lg);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .day-title {
    font-size: var(--text-xl);
    font-weight: 600;
    margin: 0;
  }
  
  .add-task-btn {
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    font-weight: 500;
  }
  
  .day-content {
    flex: 1;
    overflow: auto;
    padding: var(--spacing-md);
  }
  
  .empty-state {
    text-align: center;
    padding: var(--spacing-xl);
    color: var(--text-secondary);
  }
  
  .empty-state button {
    margin-top: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--accent);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
  }
  
  .events-section {
    margin-bottom: var(--spacing-lg);
  }
  
  .event-card {
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border-left: 4px solid var(--accent);
    margin-bottom: var(--spacing-sm);
  }
  
  .event-time {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    margin-bottom: var(--spacing-xs);
  }
  
  .event-title {
    font-weight: 500;
  }
  
  .event-description {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin-top: var(--spacing-xs);
  }
  
  .tasks-section h3 {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--spacing-md);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .task-card {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    margin-bottom: var(--spacing-sm);
    cursor: pointer;
    transition: background 0.15s;
  }
  
  .task-card:hover {
    background: var(--bg-hover);
  }
  
  .task-card.completed .task-title {
    text-decoration: line-through;
    opacity: 0.6;
  }
  
  .task-checkbox input {
    width: 20px;
    height: 20px;
    cursor: pointer;
  }
  
  .task-info {
    flex: 1;
  }
  
  .task-title {
    font-weight: 500;
  }
  
  .task-project {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin-top: 2px;
  }
  
  .edit-hint {
    opacity: 0.5;
  }
</style>
