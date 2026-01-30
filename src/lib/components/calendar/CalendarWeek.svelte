<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import CalendarHeader from './CalendarHeader.svelte';
  
  let weekDays = $derived(generateWeekDays(calendarStore.currentDate));
  
  function generateWeekDays(date: Date): Date[] {
    const start = new Date(date);
    start.setDate(start.getDate() - start.getDay());
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(start);
      d.setDate(d.getDate() + i);
      return d;
    });
  }
  
  const hours = Array.from({ length: 24 }, (_, i) => i);
  
  function getTasksForDate(date: Date): any[] {
    const dateStr = calendarStore.dateToString(date);
    return calendarStore.getTasksForDate(dateStr);
  }
  
  function formatHour(hour: number): string {
    if (hour === 0) return '12 AM';
    if (hour === 12) return '12 PM';
    if (hour < 12) return `${hour} AM`;
    return `${hour - 12} PM`;
  }
</script>

<CalendarHeader />

<div class="week-view">
  <div class="week-header">
    {#each weekDays as day, i}
      <div 
        class="week-day-header"
        class:today={day.toDateString() === new Date().toDateString()}
      >
        <span class="day-name">{['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'][i]}</span>
        <span class="day-num">{day.getDate()}</span>
      </div>
    {/each}
  </div>
  
  <div class="week-grid">
    <div class="hour-labels">
      {#each hours as hour}
        <div class="hour-label">{formatHour(hour)}</div>
      {/each}
    </div>
    
    {#each weekDays as day}
      <div class="day-column">
        {#each hours as hour}
          <div class="hour-cell">
            {#if hour === 8}
              {@const tasks = getTasksForDate(day)}
              {#each tasks as task}
                <div 
                  class="task-chip"
                  draggable="true"
                >
                  {task.title}
                </div>
              {/each}
            {/if}
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .week-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  
  .week-header {
    display: grid;
    grid-template-columns: 50px repeat(7, 1fr);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }
  
  .week-day-header {
    padding: var(--spacing-sm);
    text-align: center;
    border-left: 1px solid var(--border-color);
  }
  
  .week-day-header.today {
    background: var(--accent-light);
  }
  
  .day-name {
    display: block;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }
  
  .day-num {
    font-size: var(--text-lg);
    font-weight: 600;
  }
  
  .week-grid {
    display: grid;
    grid-template-columns: 50px repeat(7, 1fr);
    flex: 1;
    overflow: auto;
  }
  
  .hour-labels {
    border-right: 1px solid var(--border-color);
    flex-shrink: 0;
  }
  
  .hour-label {
    height: 60px;
    font-size: var(--text-xs);
    color: var(--text-secondary);
    text-align: right;
    padding-right: var(--spacing-xs);
    display: flex;
    align-items: start;
    padding-top: 2px;
  }
  
  .day-column {
    border-left: 1px solid var(--border-light);
    position: relative;
  }
  
  .hour-cell {
    height: 60px;
    border-bottom: 1px solid var(--border-light);
    padding: 2px;
    overflow: hidden;
  }
  
  .task-chip {
    font-size: 10px;
    padding: 2px 4px;
    border-radius: 2px;
    color: white;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
