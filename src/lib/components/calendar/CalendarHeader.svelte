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
      return (
        calendarStore.currentDate.getMonth() === today.getMonth() &&
        calendarStore.currentDate.getFullYear() === today.getFullYear()
      );
    } else {
      const weekStart = calendarStore.getWeekStart(calendarStore.currentDate);
      const weekEnd = new Date(weekStart);
      weekEnd.setDate(weekEnd.getDate() + 6);
      return today >= weekStart && today <= weekEnd;
    }
  });

  let showPicker = $state(false);
  let pickerYear = $state(new Date().getFullYear());

  function togglePicker() {
    pickerYear = calendarStore.currentDate.getFullYear();
    showPicker = !showPicker;
  }

  function selectPickerMonth(monthIdx: number) {
    const newDate = new Date(calendarStore.currentDate);
    newDate.setFullYear(pickerYear);
    newDate.setMonth(monthIdx);
    calendarStore.setCurrentDate(newDate);
    showPicker = false;
  }
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

    <div class="selector-container">
      {#if calendarStore.viewMode === "month"}
        <button class="month-label" onclick={togglePicker}>
          {months[calendarStore.currentDate.getMonth()]}
          {calendarStore.currentDate.getFullYear()}
          {#if !isCurrent}
            <span class="not-current-dot" title="Not current month"></span>
          {/if}
          <span class="chevron"></span>
        </button>
      {:else}
        <button class="month-label" onclick={togglePicker}>
          Week of {calendarStore.currentDate.toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
          })}
          {#if !isCurrent}
            <span class="not-current-dot" title="Not current week"></span>
          {/if}
          <span class="chevron">▼</span>
        </button>
      {/if}

      {#if showPicker}
        <div
          class="calendar-view-backdrop"
          aria-label="Close picker"
          role="button"
          tabindex="0"
          onclick={() => (showPicker = false)}
          onkeydown={(e) => e.key === "Escape" && (showPicker = false)}
        ></div>
        <div class="picker-dropdown">
          <div class="year-stepper">
            <button
              class="stepper-btn"
              aria-label="Previous year"
              onclick={() => pickerYear--}
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"><path d="m15 18-6-6 6-6" /></svg
              >
            </button>
            <span class="year-display">{pickerYear}</span>
            <button
              class="stepper-btn"
              aria-label="Next year"
              onclick={() => pickerYear++}
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"><path d="m9 18 6-6-6-6" /></svg
              >
            </button>
          </div>

          <div class="month-grid">
            {#each months as month, idx}
              <button
                class="month-btn"
                class:active={calendarStore.currentDate.getMonth() === idx &&
                  calendarStore.currentDate.getFullYear() === pickerYear}
                onclick={() => selectPickerMonth(idx)}
              >
                {month.substring(0, 3)}
              </button>
            {/each}
          </div>

          <div class="picker-footer">
            <button
              class="today-btn"
              onclick={() => {
                goToToday();
                showPicker = false;
              }}>Jump to Today</button
            >
          </div>
        </div>
      {/if}
    </div>

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
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-xs);
    font-size: var(--text-base);
    font-weight: 600;
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    min-width: 190px;
    color: var(--text-primary);
    transition: background 0.15s;
  }

  .chevron {
    font-size: 10px;
    color: var(--text-secondary);
    transition: transform 0.2s;
  }

  .selector-container {
    position: relative;
  }

  .picker-dropdown {
    position: absolute;
    top: calc(100% + var(--spacing-xs));
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    padding: var(--spacing-md);
    z-index: 100;
    width: 240px;
  }

  .year-stepper {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-md);
  }

  .stepper-btn {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-secondary);
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.1s;
  }

  .stepper-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .year-display {
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--text-primary);
  }

  .month-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-xs);
    margin-bottom: var(--spacing-md);
  }

  .month-btn {
    background: transparent;
    border: none;
    padding: var(--spacing-xs) 0;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 500;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background 0.1s;
  }

  .month-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .month-btn.active {
    background: var(--accent);
    color: white;
  }

  .picker-footer {
    display: flex;
    justify-content: stretch;
    border-top: 1px solid var(--border);
    padding-top: var(--spacing-md);
  }

  .today-btn {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: var(--spacing-sm);
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.1s;
  }

  .today-btn:hover {
    background: var(--bg-hover);
    border-color: var(--text-tertiary);
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

  /* Global backdrop for clicking outside dropdowns */
  :global(.calendar-view-backdrop) {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 99;
  }
</style>
