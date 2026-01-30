<script lang="ts">
  import { calendarStore } from '$lib/stores/calendar.svelte';
  
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
</script>

<div class="calendar-header">
  <div class="header-nav">
    <button class="nav-btn" onclick={previousMonth} aria-label="Previous month">‹</button>
    <button class="month-label" onclick={goToToday}>
      {months[calendarStore.currentDate.getMonth()]}
      {calendarStore.currentDate.getFullYear()}
    </button>
    <button class="nav-btn" onclick={nextMonth} aria-label="Next month">›</button>
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
  
  <button 
    class="close-btn" 
    onclick={() => calendarStore.close()}
    aria-label="Close calendar"
  >
    ✕
  </button>
</div>

<style>
  .calendar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }
  
  .header-nav {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }
  
  .month-label {
    font-size: var(--text-lg);
    font-weight: 600;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    min-width: 180px;
    text-align: center;
  }
  
  .month-label:hover {
    background: var(--bg-hover);
  }
  
  .nav-btn {
    background: none;
    border: none;
    font-size: var(--text-xl);
    cursor: pointer;
    padding: var(--spacing-xs) var(--spacing-md);
    border-radius: var(--radius-md);
    min-width: 44px;
  }
  
  .nav-btn:hover {
    background: var(--bg-hover);
  }
  
  .view-toggle {
    display: flex;
    gap: var(--spacing-xs);
    background: var(--bg-primary);
    padding: 3px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
  }
  
  .view-toggle button {
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: all 0.2s;
  }
  
  .view-toggle button.active {
    background: var(--accent);
    color: white;
  }
  
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-sm);
    border-radius: var(--radius-md);
    min-width: 44px;
    font-size: var(--text-lg);
  }
  
  .close-btn:hover {
    background: var(--bg-hover);
  }
</style>
