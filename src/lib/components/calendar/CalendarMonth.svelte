<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import CalendarDayCell from './CalendarDayCell.svelte';

  let calendarDays = $derived(
    calendarStore.generateCalendarDays(
      calendarStore.currentDate.getFullYear(),
      calendarStore.currentDate.getMonth()
    )
  );

  let isPortrait = $derived(uiStore.windowOrientation === 'left' || uiStore.windowOrientation === 'right');
  let weekdayLabels = $derived(
    calendarStore.generateWeekDays(calendarStore.currentDate).map((day) => day.dayName),
  );
</script>

<div class="calendar-month" class:portrait={isPortrait}>
  <div class="weekday-header">
    {#each weekdayLabels as day}
      <span class="weekday">{day}</span>
    {/each}
  </div>

  <div class="days-grid">
    {#each calendarDays as day (calendarStore.dateToString(day.date))}
      <CalendarDayCell {day} />
    {/each}
  </div>
</div>

<style>
  .calendar-month {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .weekday-header {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--bg-primary);
    z-index: 10;
  }

  .weekday {
    padding: var(--spacing-xs);
    padding-left: calc(var(--spacing-xs) + 4px); /* Extra optical adjustment so text aligns with the centered number inside the 24px circle */
    text-align: left;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .days-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    grid-template-rows: repeat(6, minmax(92px, 1fr));
    min-height: calc(100% - 31px);
    border-left: 1px solid var(--border-light);
  }

  .portrait .days-grid {
    grid-template-rows: repeat(6, minmax(70px, 1fr));
  }

  .portrait .weekday {
    font-size: 11px;
  }

  @media (max-width: 480px) {
    .days-grid {
      grid-auto-rows: minmax(80px, auto);
    }

    .weekday {
      font-size: 11px;
    }
  }
</style>
