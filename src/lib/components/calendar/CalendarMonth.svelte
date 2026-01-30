<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import CalendarDayCell from './CalendarDayCell.svelte';
  import CalendarHeader from './CalendarHeader.svelte';
  
  let calendarDays = $derived(
    calendarStore.generateCalendarDays(
      calendarStore.currentDate.getFullYear(),
      calendarStore.currentDate.getMonth()
    )
  );
</script>

<CalendarHeader />

<div class="calendar-month">
  <div class="weekday-header">
    {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
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
    overflow: auto;
  }
  
  .weekday-header {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    border-bottom: 1px solid var(--border-color);
    position: sticky;
    top: 0;
    background: var(--bg-primary);
    z-index: 10;
  }
  
  .weekday {
    padding: var(--spacing-sm);
    text-align: center;
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-secondary);
  }
  
  .days-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-auto-rows: minmax(100px, auto);
  }
  
  @media (max-width: 480px) {
    .days-grid {
      grid-auto-rows: minmax(80px, auto);
    }
    
    .weekday {
      padding: var(--spacing-xs);
      font-size: var(--text-xs);
    }
  }
</style>
