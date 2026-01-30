<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import type { TimeEntryWithTask } from '$lib/types/calendar';
  import TimeDisplay from '$lib/components/common/TimeDisplay.svelte';

  const hours = Array.from({ length: 24 }, (_, i) => i);

  let weekStart = $derived(calendarStore.getWeekStart(calendarStore.currentDate));

  let weekDays = $derived(calendarStore.generateWeekDays(weekStart));

  function getEntriesForDay(date: Date): TimeEntryWithTask[] {
    const dateStr = calendarStore.dateToString(date);
    return calendarStore.timeEntriesByDate.get(dateStr) || [];
  }

  function getEntryPosition(entry: TimeEntryWithTask): { top: number; height: number } {
    const startDate = new Date(entry.started_at * 1000);
    const endDate = new Date(entry.ended_at * 1000);

    const startMinutes = startDate.getHours() * 60 + startDate.getMinutes();
    const durationMinutes = (entry.duration_seconds / 60);

    const isPortrait = uiStore.windowOrientation === 'left' || uiStore.windowOrientation === 'right';
    const pxPerMinute = isPortrait ? 0.8 : 1;

    return {
      top: startMinutes * pxPerMinute,
      height: Math.max(durationMinutes * pxPerMinute, 20),
    };
  }

  function isToday(date: Date): boolean {
    const today = new Date();
    return date.toDateString() === today.toDateString();
  }
</script>

<div class="calendar-week" class:portrait={uiStore.windowOrientation === 'left' || uiStore.windowOrientation === 'right'}>
  <div class="time-axis">
    <div class="corner-cell"></div>
    {#each hours as hour}
      <div class="hour-label" class:minor={hour % 6 !== 0}>
        {#if hour % 6 === 0}
          {hour}:00
        {/if}
      </div>
    {/each}
  </div>

  {#each weekDays as day}
    <div class="day-column" class:today={isToday(day.date)}>
      <div class="day-header">
        <span class="day-name">{day.dayName}</span>
        <span class="day-number">{day.date.getDate()}</span>
      </div>

      <div class="day-timeline">
        {#each getEntriesForDay(day.date) as entry}
          {@const pos = getEntryPosition(entry)}
          <button
            class="time-block"
            class:selected={uiStore.calendarSelectedEntry?.id === entry.id}
            style="top: {pos.top}px; height: {pos.height}px; background-color: {entry.project_color || 'var(--accent)'}"
            onclick={() => uiStore.selectCalendarEntry(entry)}
          >
            <span class="block-task">{entry.task_title}</span>
            <span class="block-duration">
              <TimeDisplay seconds={entry.duration_seconds} format="short" />
            </span>
          </button>
        {/each}

        <div class="hour-grid">
          {#each hours as hour}
            <div class="hour-line" class:minor={hour % 6 !== 0}></div>
          {/each}
        </div>
      </div>
    </div>
  {/each}
</div>

<style>
  .calendar-week {
    display: grid;
    grid-template-columns: 50px repeat(7, 1fr);
    height: 100%;
    overflow: auto;
  }

  .calendar-week.portrait {
    grid-template-columns: 40px repeat(7, 1fr);
  }

  .time-axis {
    position: sticky;
    left: 0;
    top: 0;
    z-index: 20;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
  }

  .corner-cell {
    height: 50px;
    border-bottom: 1px solid var(--border);
  }

  .hour-label {
    height: 60px;
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    border-bottom: 1px solid transparent;
  }

  .portrait .hour-label {
    height: 48px;
    font-size: 10px;
  }

  .hour-label.minor {
    color: var(--text-tertiary);
  }

  .day-column {
    border-left: 1px solid var(--border-light);
    position: relative;
  }

  .day-column.today {
    background: color-mix(in srgb, var(--accent) 5%, transparent);
  }

  .day-header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 50px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .portrait .day-header {
    height: 40px;
    font-size: 11px;
  }

  .day-name {
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .day-number {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .day-column.today .day-number {
    background: var(--accent);
    color: white;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .day-timeline {
    position: relative;
    height: 1440px;
  }

  .portrait .day-timeline {
    height: 1152px;
  }

  .hour-grid {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .hour-line {
    height: 60px;
    border-bottom: 1px solid var(--border-light);
  }

  .portrait .hour-line {
    height: 48px;
  }

  .hour-line.minor {
    border-bottom: 1px dashed var(--border-light);
    opacity: 0.5;
  }

  .time-block {
    position: absolute;
    left: 4px;
    right: 4px;
    border-radius: var(--radius-sm);
    padding: 4px 6px;
    font-size: 11px;
    color: white;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.15s;
    border: none;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 2px;
    box-shadow: var(--shadow-sm);
  }

  .time-block:hover {
    transform: scale(1.02);
    z-index: 5;
    box-shadow: var(--shadow-md);
  }

  .time-block.selected {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    z-index: 10;
  }

  .block-task {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .block-duration {
    font-size: 10px;
    opacity: 0.9;
    font-family: var(--font-mono);
  }
</style>
