<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';

  const months = ['January', 'February', 'March', 'April', 'May', 'June',
                 'July', 'August', 'September', 'October', 'November', 'December'];

  function previousMonth() {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setMonth(newDate.getMonth() - 1);
    calendarStore.setCurrentDate(newDate);
  }

  function nextMonth() {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setMonth(newDate.getMonth() + 1);
    calendarStore.setCurrentDate(newDate);
  }

  function goToToday() {
    calendarStore.setCurrentDate(new Date());
  }

  function previousWeek() {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setDate(newDate.getDate() - 7);
    calendarStore.setCurrentDate(newDate);
  }

  function nextWeek() {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setDate(newDate.getDate() + 7);
    calendarStore.setCurrentDate(newDate);
  }

  function previousDay() {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setDate(newDate.getDate() - 1);
    calendarStore.setCurrentDate(newDate);
  }

  function nextDay() {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setDate(newDate.getDate() + 1);
    calendarStore.setCurrentDate(newDate);
  }

  function getNavFunctions() {
    switch (calendarStore.viewMode) {
      case 'week':
        return { prev: previousWeek, next: nextWeek };
      case 'day':
        return { prev: previousDay, next: nextDay };
      default:
        return { prev: previousMonth, next: nextMonth };
    }
  }

  let navFunctions = $derived(getNavFunctions());
</script>

<div class="calendar-header">
  <div class="header-nav">
    <button class="nav-btn" onclick={navFunctions.prev} aria-label="Previous">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m15 18-6-6 6-6" />
      </svg>
    </button>

    {#if calendarStore.viewMode === 'month'}
      <button class="month-label" onclick={goToToday}>
        {months[calendarStore.currentDate.getMonth()]}
        {calendarStore.currentDate.getFullYear()}
      </button>
    {:else if calendarStore.viewMode === 'week'}
      <button class="month-label" onclick={goToToday}>
        Week of {calendarStore.currentDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
      </button>
    {:else}
      <button class="month-label" onclick={goToToday}>
        {calendarStore.currentDate.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' })}
      </button>
    {/if}

    <button class="nav-btn" onclick={navFunctions.next} aria-label="Next">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="m9 18 6-6-6-6" />
      </svg>
    </button>
  </div>

  <div class="view-toggle">
    <button
      class:active={calendarStore.viewMode === 'month'}
      onclick={() => calendarStore.setViewMode('month')}
    >Month</button>
    <button
      class:active={calendarStore.viewMode === 'week'}
      onclick={() => calendarStore.setViewMode('week')}
    >Week</button>
    <button
      class:active={calendarStore.viewMode === 'day'}
      onclick={() => calendarStore.setViewMode('day')}
    >Day</button>
  </div>
</div>

<style>
  .calendar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
  }

  .header-nav {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .month-label {
    font-size: var(--text-base);
    font-weight: 600;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    min-width: 180px;
    text-align: center;
    color: var(--text-primary);
  }

  .month-label:hover {
    background: var(--bg-hover);
  }

  .nav-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-sm);
    border-radius: var(--radius-md);
    min-width: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .nav-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .view-toggle {
    display: flex;
    gap: var(--spacing-xs);
    background: var(--bg-primary);
    padding: 3px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .view-toggle button {
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .view-toggle button.active {
    background: var(--accent);
    color: white;
  }
</style>
