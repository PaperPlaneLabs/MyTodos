<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import type { CalendarDay, CalendarTask } from "$lib/types/calendar";

  let { day } = $props<{ day: CalendarDay }>();

  const taskLimit = 3;
  const eventLimit = 1;

  function formatDate(date: Date): string {
    return calendarStore.dateToString(date);
  }

  function getDayAriaLabel(date: Date): string {
    return date.toLocaleDateString("en-US", {
      weekday: "long",
      month: "long",
      day: "numeric",
      year: "numeric",
    });
  }

  function handleDayClick() {
    calendarStore.setSelectedDate(day.date);
    // Removed automatic modal opening to favor the new task list at the bottom
  }

  function handleTaskClick(e: Event, task: CalendarTask) {
    e.stopPropagation();
    uiStore.openTaskModal({ taskId: task.id, deadline: formatDate(day.date) });
  }

  function handleAddTaskClick(e: Event) {
    e.stopPropagation();
    uiStore.openTaskModal({ deadline: formatDate(day.date) });
  }

  function handleTaskDragStart(e: DragEvent, task: CalendarTask) {
    e.dataTransfer?.setData("task-id", String(task.id));
    e.dataTransfer!.effectAllowed = "move";
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    const taskId = e.dataTransfer?.getData("task-id");
    if (taskId && day.isCurrentMonth) {
      const newDeadline = formatDate(day.date);
      await calendarStore.updateTaskDeadline(parseInt(taskId, 10), newDeadline);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = "move";
  }

  function getTaskColor(task: CalendarTask): string {
    if (task.project_id) {
      const project = projectStore.projects.find(
        (p) => p.id === task.project_id,
      );
      if (project?.color) {
        return project.color;
      }
    }
    return "var(--accent)";
  }

  let totalCount = $derived(day.tasks.length + day.events.length);
  let isDragOver = $state(false);
  let isWeekend = $derived(
    day.date.getDay() === 0 || day.date.getDay() === 6
  );

  // Unique project colors for the dot indicators (month view)
  let projectDots = $derived.by(() => {
    const seen = new Set<string>();
    const colors: string[] = [];
    for (const task of day.tasks) {
      const color = getTaskColor(task);
      if (!seen.has(color)) {
        seen.add(color);
        colors.push(color);
      }
    }
    // Add a generic accent dot for calendar events
    if (day.events.length > 0 && !seen.has('var(--accent)')) {
      colors.push('var(--accent)');
    }
    return colors;
  });

  let extraDotCount = $derived(Math.max(0, projectDots.length - 3));

  function handleDragEnter(e: DragEvent) {
    if (!day.isCurrentMonth) return;
    e.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    // Only clear if leaving the cell entirely (not entering a child)
    const target = e.currentTarget as HTMLElement;
    if (!target.contains(e.relatedTarget as Node)) {
      isDragOver = false;
    }
  }
</script>

<div
  class="day-cell"
  class:today={day.isToday}
  class:other-month={!day.isCurrentMonth}
  class:selected={day.isSelected}
  class:has-content={totalCount > 0}
  class:drag-over={isDragOver}
  class:weekend={isWeekend}
  ondragover={handleDragOver}
  ondragenter={handleDragEnter}
  ondragleave={handleDragLeave}
  ondrop={(e) => { isDragOver = false; handleDrop(e); }}
  onclick={handleDayClick}
  aria-label={getDayAriaLabel(day.date)}
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleDayClick();
    }
  }}
>
  <div class="day-header-row">
    <span class="day-number" class:today-num={day.isToday}
      >{day.date.getDate()}</span
    >
    {#if day.isCurrentMonth}
      <button
        type="button"
        class="add-task-btn"
        onclick={handleAddTaskClick}
        aria-label="Add task for this day">+</button
      >
    {/if}
  </div>

  {#if totalCount > 0}
    <div class="day-content">
      {#if calendarStore.viewMode === "month"}
        <div class="project-dots">
          {#each projectDots.slice(0, 3) as color}
            <span class="project-dot" style="background-color: {color}"></span>
          {/each}
          {#if extraDotCount > 0}
            <span class="dot-overflow">+{extraDotCount}</span>
          {/if}
        </div>
      {:else}
        {#each day.events.slice(0, eventLimit) as event}
          <div
            class="event-chip"
            style="border-left-color: {event.color || 'var(--accent)'}"
          >
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
            style="--task-color: {getTaskColor(task)}"
          >
            {#if task.completed}
              <span class="check-icon">✓</span>
            {/if}
            <span class="task-title">{task.title}</span>
          </button>
        {/each}

        {#if day.tasks.length > taskLimit || day.events.length > eventLimit}
          <span class="more-tasks">
            +{Math.max(0, day.tasks.length - taskLimit) +
              Math.max(0, day.events.length - eventLimit)} more
          </span>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .day-cell {
    min-height: 100px;
    min-width: 0; /* Prevents long task contents from expanding grid columns */
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
    background: color-mix(in srgb, var(--accent) 6%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .day-cell.weekend:not(.today) {
    background: color-mix(in srgb, var(--text-tertiary) 4%, var(--bg-primary));
  }

  .day-cell.selected {
    box-shadow: inset 0 0 0 2px var(--accent);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }

  .day-cell.drag-over {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
    box-shadow: inset 0 0 0 2px color-mix(in srgb, var(--accent) 40%, transparent);
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
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .day-number.today-num {
    background: var(--accent);
    color: var(--accent-contrast);
    font-weight: 700;
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
    opacity: 0;
  }

  .day-cell:hover .add-task-btn {
    opacity: 1;
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
    color: var(--text-primary);
    border: 1px solid color-mix(in srgb, var(--task-color) 30%, var(--border));
    border-left: 3px solid var(--task-color);
    background: color-mix(in srgb, var(--task-color) 14%, var(--bg-primary));
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



  /* Project dot indicators (month view) */
  .project-dots {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    margin-top: 4px;
  }

  .project-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
  }

  .dot-overflow {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-tertiary);
    line-height: 1;
  }

  @media (max-width: 480px) {
    .day-cell {
      min-height: 80px;
    }



    .task-chip,
    .more-tasks {
      font-size: 10px;
    }
  }
</style>
