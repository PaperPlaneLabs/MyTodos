<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { projectStore } from '$lib/stores/projects.svelte';
  import type { CalendarDay, CalendarTask } from '$lib/types/calendar';
  
  let { day } = $props<{ day: CalendarDay }>();
  
  const taskLimit = 3;
  const eventLimit = 1;
  
  function formatDate(date: Date): string {
    return calendarStore.dateToString(date);
  }
  
  function handleDayClick() {
    calendarStore.setSelectedDate(day.date);
    uiStore.openTaskModal({ deadline: formatDate(day.date) });
  }
  
  function handleTaskClick(e: Event, task: CalendarTask) {
    e.stopPropagation();
    uiStore.openTaskModal({ task, deadline: formatDate(day.date) });
  }

  function handleAddTaskClick(e: Event) {
    e.stopPropagation();
    uiStore.openTaskModal({ deadline: formatDate(day.date) });
  }
  
  function handleTaskDragStart(e: DragEvent, task: CalendarTask) {
    e.dataTransfer?.setData('task-id', String(task.id));
    e.dataTransfer!.effectAllowed = 'move';
  }
  
  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    const taskId = e.dataTransfer?.getData('task-id');
    if (taskId && day.isCurrentMonth) {
      const newDeadline = formatDate(day.date);
      await calendarStore.updateTaskDeadline(parseInt(taskId, 10), newDeadline);
    }
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = 'move';
  }
  
  function getTaskColor(task: CalendarTask): string {
    if (task.project_id) {
      const project = projectStore.projects.find((p) => p.id === task.project_id);
      if (project?.color) {
        return project.color;
      }
    }
    return 'var(--accent)';
  }
</script>

<div 
  class="day-cell"
  class:today={day.isToday}
  class:other-month={!day.isCurrentMonth}
  class:selected={day.isSelected}
  class:has-content={day.tasks.length > 0 || day.events.length > 0}
  ondragover={handleDragOver}
  ondrop={handleDrop}
  onclick={handleDayClick}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleDayClick();
    }
  }}
>
  <div class="day-header-row">
    <span class="day-number">{day.date.getDate()}</span>
    {#if day.isCurrentMonth}
      <button
        class="add-task-btn"
        onclick={handleAddTaskClick}
        aria-label="Add task for this day"
      >+</button>
    {/if}
  </div>
  
  {#if day.tasks.length > 0 || day.events.length > 0}
    <div class="day-content">
      {#each day.events.slice(0, eventLimit) as event}
        <div class="event-chip" style="border-left-color: {event.color || 'var(--accent)'}">
          <span class="event-title">{event.title}</span>
        </div>
      {/each}

      {#each day.tasks.slice(0, taskLimit) as task}
        <button
          type="button"
          class="task-chip"
          class:completed={task.completed}
          draggable="true"
          ondragstart={(e) => handleTaskDragStart(e, task)}
          onclick={(e) => handleTaskClick(e, task)}
          style="background-color: {getTaskColor(task)}"
        >
          {#if task.completed}
            <span class="check-icon">✓</span>
          {/if}
          <span class="task-title">{task.title}</span>
        </button>
      {/each}
      
      {#if day.tasks.length > taskLimit || day.events.length > eventLimit}
        <span class="more-tasks">
          +{Math.max(0, day.tasks.length - taskLimit) + Math.max(0, day.events.length - eventLimit)} more
        </span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .day-cell {
    min-height: 100px;
    padding: var(--spacing-xs);
    border: 1px solid var(--border-light);
    background: var(--bg-primary);
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
  }
  
  .day-cell:hover {
    background: var(--bg-hover);
  }
  
  .day-cell.today {
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }
  
  .day-cell.selected {
    border-color: var(--accent);
    border-width: 2px;
  }
  
  .day-cell.other-month {
    opacity: 0.4;
    background: var(--bg-secondary);
  }
  
  .day-cell:not(.other-month):hover {
    border-color: var(--accent);
  }
  
  .day-number {
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--text-primary);
  }

  .day-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-xs);
    gap: var(--spacing-xs);
  }

  .add-task-btn {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--bg-secondary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    font-size: 14px;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .add-task-btn:hover {
    background: var(--accent-light);
    color: var(--accent);
    border-color: var(--accent);
  }
  
  .today .day-number {
    color: var(--accent);
    font-weight: 700;
  }
  
  .day-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow: hidden;
  }

  .event-chip {
    font-size: 10px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    border-left: 3px solid var(--accent);
    color: var(--text-primary);
  }

  .event-title {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  
  .task-chip {
    font-size: 11px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    color: white;
    border: none;
    display: flex;
    align-items: center;
    gap: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: grab;
    text-align: left;
    transition: transform 0.1s;
  }
  
  .task-chip:active {
    cursor: grabbing;
    transform: scale(0.98);
  }
  
  .task-chip.completed {
    opacity: 0.6;
    text-decoration: line-through;
  }
  
  .task-title {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .check-icon {
    flex-shrink: 0;
    font-size: 10px;
  }
  
  .more-tasks {
    font-size: 11px;
    color: var(--text-secondary);
    padding: 2px 4px;
  }
  
  @media (max-width: 480px) {
    .day-cell {
      min-height: 80px;
    }
    
    .task-chip, .more-tasks {
      font-size: 10px;
    }
  }
</style>
