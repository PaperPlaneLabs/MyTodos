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

  function goToToday() {
    calendarStore.setCurrentDate(new Date());
  }

  function getNavFunctions() {
    switch (calendarStore.viewMode) {
      case "week":
        return { prev: previousWeek, next: nextWeek };
      case "deadline":
        return null;
      default:
        return { prev: previousMonth, next: nextMonth };
    }
  }

  let navFunctions = $derived(getNavFunctions());
  let isPortrait = $derived(
    uiStore.windowOrientation === "left" ||
      uiStore.windowOrientation === "right",
  );
  let activeViewIndex = $derived(
    calendarStore.viewMode === "month"
      ? 0
      : calendarStore.viewMode === "week"
        ? 1
        : 2,
  );
  let canUsePicker = $derived(calendarStore.viewMode !== "deadline");

  let isCurrent = $derived.by(() => {
    const today = new Date();

    if (calendarStore.viewMode === "deadline") {
      return true;
    }

    if (calendarStore.viewMode === "month") {
      return (
        calendarStore.currentDate.getMonth() === today.getMonth() &&
        calendarStore.currentDate.getFullYear() === today.getFullYear()
      );
    }

    const weekStart = calendarStore.getWeekStart(calendarStore.currentDate);
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekEnd.getDate() + 6);
    return today >= weekStart && today <= weekEnd;
  });

  let showPicker = $state(false);
  let pickerYear = $state(new Date().getFullYear());
  const pickerId = `calendar-month-picker-${Math.random().toString(36).slice(2, 10)}`;

  function togglePicker() {
    if (!canUsePicker) return;
    pickerYear = calendarStore.currentDate.getFullYear();
    showPicker = !showPicker;
  }

  function closePicker() {
    showPicker = false;
  }

  function selectPickerMonth(monthIdx: number) {
    if (!canUsePicker) return;
    const newDate = new Date(calendarStore.currentDate);
    newDate.setFullYear(pickerYear);
    newDate.setMonth(monthIdx);
    calendarStore.setCurrentDate(newDate);
    closePicker();
  }

  $effect(() => {
    if (!canUsePicker && showPicker) {
      showPicker = false;
    }
  });
</script>

<div class="calendar-header">
  <div class="header-nav">
    {#if navFunctions}
      <button
        type="button"
        class="nav-btn"
        onclick={navFunctions.prev}
        aria-label="Previous"
      >
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
    {/if}

    <div class="selector-container">
      {#if calendarStore.viewMode === "month"}
        <button
          type="button"
          class="month-label"
          aria-controls={pickerId}
          aria-expanded={showPicker}
          aria-haspopup="dialog"
          onclick={togglePicker}
        >
          {months[calendarStore.currentDate.getMonth()]}
          {calendarStore.currentDate.getFullYear()}
          {#if !isCurrent}
            <span class="not-current-dot" title="Not current month"></span>
          {/if}
          <span class="chevron"></span>
        </button>
      {:else if calendarStore.viewMode === "week"}
        <button
          type="button"
          class="month-label"
          aria-controls={pickerId}
          aria-expanded={showPicker}
          aria-haspopup="dialog"
          onclick={togglePicker}
        >
          Week of {calendarStore.currentDate.toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
          })}
          {#if !isCurrent}
            <span class="not-current-dot" title="Not current week"></span>
          {/if}
          <span class="chevron"></span>
        </button>
      {:else}
        <div class="month-label static">
          <span>Upcoming Deadlines</span>
          <span class="deadline-subtitle">Open tasks with deadlines</span>
        </div>
      {/if}

      {#if showPicker}
        <button
          type="button"
          class="calendar-view-backdrop"
          aria-label="Close calendar month picker"
          onclick={closePicker}
        ></button>
        <div
          id={pickerId}
          class="picker-dropdown"
          role="dialog"
          aria-modal="false"
          aria-label="Choose month and year"
        >
          <div class="year-stepper">
            <button
              type="button"
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
                stroke-width="2"
              >
                <path d="m15 18-6-6 6-6" />
              </svg>
            </button>
            <span class="year-display">{pickerYear}</span>
            <button
              type="button"
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
                stroke-width="2"
              >
                <path d="m9 18 6-6-6-6" />
              </svg>
            </button>
          </div>

          <div class="month-grid">
            {#each months as month, idx}
              <button
                type="button"
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
              type="button"
              class="today-btn"
              onclick={() => {
                goToToday();
                closePicker();
              }}
            >
              Jump to Today
            </button>
          </div>
        </div>
      {/if}
    </div>

    {#if navFunctions}
      <button
        type="button"
        class="nav-btn"
        onclick={navFunctions.next}
        aria-label="Next"
      >
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
    {/if}
  </div>

  <div class="view-toggle" style={`--active-index: ${activeViewIndex};`}>
    <div class="active-bg"></div>
    <button
      type="button"
      class:active={calendarStore.viewMode === "month"}
      aria-pressed={calendarStore.viewMode === "month"}
      onclick={() => calendarStore.setViewMode("month")}
      title="Month View"
    >
      {isPortrait ? "M" : "Month"}
    </button>
    <button
      type="button"
      class:active={calendarStore.viewMode === "week"}
      aria-pressed={calendarStore.viewMode === "week"}
      onclick={() => calendarStore.setViewMode("week")}
      title="Week View"
    >
      {isPortrait ? "W" : "Week"}
    </button>
    <button
      type="button"
      class:active={calendarStore.viewMode === "deadline"}
      aria-pressed={calendarStore.viewMode === "deadline"}
      onclick={() => calendarStore.setViewMode("deadline")}
      title="Deadline View"
    >
      {isPortrait ? "DL" : "Deadline"}
    </button>
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
    gap: var(--spacing-md);
  }

  .header-nav {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .selector-container {
    position: relative;
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

  .month-label:hover {
    background: var(--bg-hover);
  }

  .month-label.static {
    flex-direction: column;
    align-items: flex-start;
    cursor: default;
    min-width: 220px;
  }

  .month-label.static:hover {
    background: none;
  }

  .deadline-subtitle {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-tertiary);
  }

  .chevron {
    width: 8px;
    height: 8px;
    border-right: 1.5px solid currentColor;
    border-bottom: 1.5px solid currentColor;
    color: var(--text-secondary);
    transform: rotate(45deg) translateY(-1px);
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
    color: var(--accent-contrast);
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
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    background: var(--bg-primary);
    padding: 3px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    min-width: 220px;
  }

  .active-bg {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 3px;
    width: calc((100% - 6px) / 3);
    background: var(--accent);
    border-radius: var(--radius-sm);
    transform: translateX(calc(var(--active-index, 0) * 100%));
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    z-index: 0;
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
  }

  .view-toggle button.active {
    color: var(--accent-contrast);
  }

  :global(.calendar-view-backdrop) {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 99;
    border: none;
    padding: 0;
    background: transparent;
  }

  @media (max-width: 860px) {
    .calendar-header {
      flex-direction: column;
      align-items: stretch;
    }

    .header-nav {
      justify-content: center;
    }

    .view-toggle {
      width: 100%;
      min-width: 0;
    }
  }
</style>
