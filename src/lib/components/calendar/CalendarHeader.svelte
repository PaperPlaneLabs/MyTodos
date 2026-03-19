<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";

  const months = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];

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

  function getNavFunctions() {
    switch (calendarStore.viewMode) {
      case "week":
        return { prev: previousWeek, next: nextWeek };
      default:
        return { prev: previousMonth, next: nextMonth };
    }
  }

  let navFunctions = $derived(getNavFunctions());
  let isPortrait = $derived(
    uiStore.windowOrientation === "left" ||
      uiStore.windowOrientation === "right",
  );

  let isCurrent = $derived.by(() => {
    const today = new Date();
    if (calendarStore.viewMode === "month") {
      return calendarStore.currentDate.getMonth() === today.getMonth() &&
             calendarStore.currentDate.getFullYear() === today.getFullYear();
    } else {
      // Very simple week check: is today within the current week's range?
      const weekStart = calendarStore.getWeekStart(calendarStore.currentDate);
      const weekEnd = new Date(weekStart);
      weekEnd.setDate(weekEnd.getDate() + 6);
      return today >= weekStart && today <= weekEnd;
    }
  });
</script>

<div class="calendar-header">
  <div class="header-nav">
    <button class="nav-btn" onclick={navFunctions.prev} aria-label="Previous">
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="m15 18-6-6 6-6" />
      </svg>
    </button>

    {#if calendarStore.viewMode === "month"}
      <button class="month-label" onclick={goToToday}>
        {months[calendarStore.currentDate.getMonth()]}
        {calendarStore.currentDate.getFullYear()}
        {#if !isCurrent}
          <span class="not-current-dot" title="Not current month"></span>
        {/if}
      </button>
    {:else}
      <button class="month-label" onclick={goToToday}>
        Week of {calendarStore.currentDate.toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
        })}
        {#if !isCurrent}
          <span class="not-current-dot" title="Not current week"></span>
        {/if}
      </button>
    {/if}

    <button class="nav-btn" onclick={navFunctions.next} aria-label="Next">
      <svg
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="m9 18 6-6-6-6" />
      </svg>
    </button>
  </div>

  <div class="view-toggle" class:is-week={calendarStore.viewMode === "week"}>
    <div class="active-bg"></div>
    <button
      class:active={calendarStore.viewMode === "month"}
      onclick={() => calendarStore.setViewMode("month")}
      title="Month View">{isPortrait ? "M" : "Month"}</button
    >
    <button
      class:active={calendarStore.viewMode === "week"}
      onclick={() => calendarStore.setViewMode("week")}
      title="Week View">{isPortrait ? "W" : "Week"}</button
    >
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
    position: relative;
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

  .not-current-dot {
    position: absolute;
    bottom: 2px;
    left: 50%;
    transform: translateX(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background-color: var(--accent);
  }

  .month-label:hover {
    background: var(--bg-hover);
  }

  .nav-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    width: 36px;
    height: 36px;
    border-radius: 50%;
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
    position: relative;
    display: flex;
    gap: var(--spacing-xs);
    background: var(--bg-primary);
    padding: 3px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .active-bg {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 3px;
    width: calc(50% - 4px); /* Account for gap and padding */
    background: var(--accent);
    border-radius: var(--radius-sm);
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 0;
  }

  .view-toggle.is-week .active-bg {
    transform: translateX(calc(100% + 2px));
  }

  .view-toggle button {
    position: relative;
    z-index: 1;
    padding: var(--spacing-xs) var(--spacing-md);
    border: none;
    background: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    transition: color 0.2s;
    flex: 1;
  }

  .view-toggle button.active {
    color: white;
  }
</style>
