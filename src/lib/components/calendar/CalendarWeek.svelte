<script lang="ts">
  import { onMount, tick } from "svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import type { Task, CalendarEvent } from "$lib/services/db";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";

  // Derived state
  let weekStart = $derived(
    calendarStore.getWeekStart(calendarStore.currentDate),
  );
  let weekDays = $derived(calendarStore.generateWeekDays(weekStart));
  let isPortrait = $derived(
    uiStore.windowOrientation === "left" ||
      uiStore.windowOrientation === "right",
  );

  // Helpers
  function getProjectColor(projectId: number | null): string {
    if (!projectId) return "var(--text-tertiary)";
    const project = projectStore.projects.find((p) => p.id === projectId);
    return project?.color || "var(--text-tertiary)";
  }

  function formatTimeFromDate(date: Date): string {
    return date.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: true,
    });
  }

  function formatDocsTime(deadline: string | null): string | null {
    if (!deadline || !deadline.includes("T")) return null;
    const timePart = deadline.split("T")[1];
    if (!timePart) return null;
    const [hours, minutes] = timePart.split(":");
    const date = new Date();
    date.setHours(parseInt(hours), parseInt(minutes));
    return formatTimeFromDate(date);
  }

  function getSortableTime(deadline: string | null): string {
    if (!deadline || !deadline.includes("T")) return "99:99"; // Sort to bottom if using string sort, but we handle separation logic
    return deadline.split("T")[1];
  }

  function handleTaskClick(task: Task) {
    uiStore.openTaskModal({ taskId: task.id });
  }

  type ScheduleItem = {
    type: "task" | "event";
    id: number | string;
    title: string;
    color: string;
    time: string | null;
    isAllDay: boolean;
    sortTime: string; // HH:MM for sorting
    data: Task | CalendarEvent;
  };

  function getDayItems(date: Date) {
    const dateStr = calendarStore.dateToString(date);
    const tasks = calendarStore.getTasksForDate(dateStr);
    const events = calendarStore.getEventsForDate(dateStr);

    const items: ScheduleItem[] = [];

    // Process Tasks
    tasks.forEach((task) => {
      const hasTime = task.deadline && task.deadline.includes("T");
      items.push({
        type: "task",
        id: task.id,
        title: task.title,
        color: getProjectColor(task.project_id ?? null),
        time: hasTime ? formatDocsTime(task.deadline ?? null) : null,
        isAllDay: !hasTime,
        sortTime: hasTime ? getSortableTime(task.deadline ?? null) : "23:59",
        data: task,
      });
    });

    // Process Events
    events.forEach((event) => {
      // Assuming event structure. If we don't have start time in event object from getEventsForDate,
      // checking how it was used in Day view: {event.is_all_day ? 'All day' : ''}
      // We'll treat all events without explicit start time logic as all-day or check if we have properties.
      // Based on types seen in other files, events usually have title, color, is_all_day.
      // We'll place them at bottom if all_day.
      items.push({
        type: "event",
        id: event.id || Math.random().toString(), // fallback id
        title: event.title,
        color: event.color || "var(--accent)",
        time: event.is_all_day ? null : "Event", // Placeholder if we had time
        isAllDay: !!event.is_all_day,
        sortTime: "23:59", // Default to bottom for events if no precise time
        data: event,
      });
    });

    // Sort: Timed items first (sorted by time), then All-Day items (as requested by user "at the bottom")
    const timedItems = items.filter((i) => !i.isAllDay);
    const allDayItems = items.filter((i) => i.isAllDay);

    timedItems.sort((a, b) => a.sortTime.localeCompare(b.sortTime));

    // Sort all day items by type then title
    allDayItems.sort((a, b) => {
      if (a.type !== b.type) return a.type === "event" ? -1 : 1; // Events first in bottom section? Or doesn't matter.
      return a.title.localeCompare(b.title);
    });

    return [...timedItems, ...allDayItems];
  }

  function isToday(date: Date): boolean {
    const today = new Date();
    return date.toDateString() === today.toDateString();
  }
</script>

<div class="schedule-view" class:portrait={isPortrait}>
  {#each weekDays as day}
    {@const items = getDayItems(day.date)}
    <div class="day-row" class:today={isToday(day.date)}>
      <!-- Left Column: Date -->
      <div class="date-column">
        <span class="day-name">{day.dayName}</span>
        <div class="day-number-wrapper">
          <span class="day-number">{day.date.getDate()}</span>
        </div>
      </div>

      <!-- Right Column: Schedule Items -->
      <div class="items-column">
        {#if items.length === 0}
          <div class="empty-slot"></div>
        {:else}
          {#each items as item}
            {#if item.type === "task"}
              <!-- Task Item -->
              <button
                class="schedule-item task"
                onclick={() => handleTaskClick(item.data as Task)}
                type="button"
              >
                <div class="time-col">
                  {#if item.time}
                    <span class="item-time">{item.time}</span>
                  {:else}
                    <span class="item-time all-day">All Day</span>
                  {/if}
                </div>
                <div
                  class="color-bar"
                  style="background-color: {item.color}"
                ></div>
                <div class="content-col">
                  <span
                    class="item-title"
                    class:completed={(item.data as Task).completed}
                  >
                    {item.title}
                  </span>
                </div>
                {#if (item.data as Task).completed}
                  <span class="check-icon">✓</span>
                {/if}
              </button>
            {:else}
              <!-- Event Item -->
              <div class="schedule-item event">
                <div class="time-col">
                  <span class="item-time all-day">All Day</span>
                </div>
                <div
                  class="color-bar event-bar"
                  style="background-color: {item.color}"
                ></div>
                <div class="content-col">
                  <span class="item-title">{item.title}</span>
                </div>
              </div>
            {/if}
          {/each}
        {/if}
      </div>
    </div>
  {/each}
</div>

<style>
  .schedule-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    background: var(--bg-primary);
  }

  .day-row {
    display: grid;
    grid-template-columns: 60px 1fr;
    border-bottom: 1px solid var(--border-light);
    min-height: 80px;
  }

  .day-row:last-child {
    border-bottom: none;
  }

  .date-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: var(--spacing-md);
    gap: 4px;
    border-right: 1px solid var(--border-light);
    background: var(--bg-secondary);
  }

  .day-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
  }

  .day-number-wrapper {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
  }

  .day-number {
    font-size: 18px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .day-row.today .day-number-wrapper {
    background: var(--accent);
  }

  .day-row.today .day-number {
    color: white;
  }

  .day-row.today .day-name {
    color: var(--accent);
  }

  .items-column {
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .empty-slot {
    height: 20px;
  }

  .schedule-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: none;
    text-align: left;
    width: 100%;
    cursor: pointer;
    transition: background 0.1s;
  }

  .schedule-item:hover {
    background: var(--bg-hover);
  }

  .schedule-item.event {
    cursor: default;
  }

  .time-col {
    width: 65px;
    flex-shrink: 0;
    text-align: right;
  }

  .item-time {
    font-size: 12px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .item-time.all-day {
    font-size: 10px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-weight: 600;
  }

  .color-bar {
    width: 4px;
    height: 24px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  /* Distinguish events with a dot or different style if needed, but bar is fine */
  .color-bar.event-bar {
    border-radius: 2px;
    width: 4px;
  }

  .content-col {
    flex: 1;
    overflow: hidden;
    display: flex;
    align-items: center;
  }

  .item-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-title.completed {
    text-decoration: line-through;
    color: var(--text-tertiary);
  }

  .check-icon {
    color: var(--success);
    font-size: 14px;
    margin-left: auto;
  }
</style>
