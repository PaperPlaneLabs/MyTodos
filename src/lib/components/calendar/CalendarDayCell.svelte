<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import type { CalendarDay, CalendarTask } from '$lib/types/calendar';
  
  let { day } = $props<{ day: CalendarDay }>();
  
  const taskLimit = 3;
  
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
  
  function handleTaskDragStart(e: DragEvent, task: CalendarTask) {
    e.dataTransfer?.setData('task-id', String(task.id));
    e.dataTransfer!.effectAllowed = 'move';
  }
  
  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    const taskId = e.dataTransfer?.getData('task-id');
    if (taskId && day.isCurrentMonth) {
      const newDeadline = formatDate(day.date);
      calendarStore.updateTaskDeadline(parseInt(taskId), newDeadline);
    }
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = 'move';
  }
  
  function formatProjectColor(color: string | undefined): string {
    return color ? `var(--color-${color})` : 'var(--accent)';
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
  onkeydown={(e) => e.key === 'Enter' && handleDayClick()}
>
  <span class="day-number">{day.date.getDate()}</span>
  
  {#if day.tasks.length > 0 || day.events.length > 0}
    <div class="day-content">
      {#each day.tasks.slice(0, taskLimit) as task}
        <div 
          class="task-chip"
          class:completed={task.completed}
          draggable="true"
          ondragstart={(e) => handleTaskDragStart(e, task)}
          onclick={(e) => handleTaskClick(e, task)}
          style="background-color: {formatProjectColor(task.project_id ? undefined : undefined)}"
        >
          {#if task.completed}
            <span class="check-icon">✓</span>
          {/if}
          <span class="task-title">{task.title}</span>
        </div>
      {/each}
      
      {#if day.tasks.length > taskLimit}
        <span class="more-tasks">+{day.tasks.length - taskLimit} more</span>
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
    display: block;
    margin-bottom: var(--spacing-xs);
    color: var(--text-primary);
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
  
  .task-chip {
    font-size: 11px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    color: white;
    display: flex;
    align-items: center;
    gap: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: grab;
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
