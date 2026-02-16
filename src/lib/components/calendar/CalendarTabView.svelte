<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { db } from "$lib/services/db";
  import CalendarHeader from "./CalendarHeader.svelte";
  import CalendarMonth from "./CalendarMonth.svelte";
  import CalendarWeek from "./CalendarWeek.svelte";
  import CalendarDay from "./CalendarDay.svelte";
  import TimeEntryPanel from "./TimeEntryPanel.svelte";
  import DayTaskList from "./DayTaskList.svelte";

  let isInitializing = $state(true);

  onMount(async () => {
    try {
      await calendarStore.ensureCurrentRangeLoaded();
      const orientation = await db.window.getOrientation();
      uiStore.setWindowOrientation(
        orientation.side as "left" | "right" | "center",
      );
    } catch (e) {
      console.error("Failed to load calendar:", e);
    } finally {
      isInitializing = false;
    }
  });
</script>

<div class="calendar-tab" transition:fade={{ duration: 200 }}>
  <header class="calendar-header-panel">
    <button class="back-btn" onclick={() => uiStore.closeCalendarView()}>
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
      <span>Back</span>
    </button>
    <h2>Calendar</h2>
  </header>

  {#if isInitializing && calendarStore.isLoading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading calendar...</p>
    </div>
  {:else}
    <div
      class="calendar-body"
      class:portrait={uiStore.windowOrientation === "left" ||
        uiStore.windowOrientation === "right"}
    >
      <div class="calendar-main">
        <CalendarHeader />

        <div class="calendar-content">
          {#if calendarStore.viewMode === "month"}
            <CalendarMonth />
          {:else if calendarStore.viewMode === "week"}
            <CalendarWeek />
          {:else}
            <CalendarDay />
          {/if}
        </div>
      </div>

      {#if calendarStore.viewMode !== "day"}
        <div class="day-list-panel">
          <DayTaskList />
        </div>
      {/if}

      {#if uiStore.calendarSelectedEntry}
        <div class="entry-panel-wrapper">
          <TimeEntryPanel entry={uiStore.calendarSelectedEntry} />
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .calendar-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
    overflow: hidden;
  }

  .calendar-header-panel {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    font-size: var(--text-sm);
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .back-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .calendar-header-panel h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .calendar-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .calendar-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .calendar-content {
    flex: 1;
    overflow: auto;
    padding: var(--spacing-md);
  }

  .calendar-body.portrait {
    flex-direction: column;
  }

  .entry-panel-wrapper {
    width: 320px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
  }

  .calendar-body.portrait .entry-panel-wrapper {
    width: 100%;
    max-height: 40%;
    border-top: 1px solid var(--border);
    border-left: none;
  }

  .day-list-panel {
    width: 320px;
    flex-shrink: 0;
    border-left: 1px solid var(--border);
    background: var(--bg-secondary);
  }

  .calendar-body.portrait .day-list-panel {
    width: 100%;
    max-height: 40%;
    border-top: 1px solid var(--border);
    border-left: none;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--spacing-md);
    color: var(--text-secondary);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
