<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { db } from "$lib/services/db";
  import CalendarHeader from "./CalendarHeader.svelte";
  import CalendarMonth from "./CalendarMonth.svelte";
  import CalendarWeek from "./CalendarWeek.svelte";
  import CalendarDeadline from "./CalendarDeadline.svelte";
  import CalendarSkeleton from "./CalendarSkeleton.svelte";
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
    <CalendarSkeleton />
  {:else}
    <div
      class="calendar-body"
      class:portrait={uiStore.windowOrientation === "left" ||
        uiStore.windowOrientation === "right"}
    >
      <div class="calendar-main">
        <CalendarHeader />

        <div class="calendar-content slim-scroll">
          {#key calendarStore.viewMode}
            <div
              class="view-transition-wrapper"
              in:fade={{ duration: 200, delay: 100 }}
              out:fade={{ duration: 100 }}
            >
              {#if calendarStore.viewMode === "month"}
                <CalendarMonth />
              {:else if calendarStore.viewMode === "week"}
                <CalendarWeek />
              {:else}
                <CalendarDeadline />
              {/if}
            </div>
          {/key}
        </div>
      </div>

      {#if calendarStore.viewMode !== "deadline"}
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
    overflow-y: auto;
    overflow-x: hidden;
    position: relative;
  }

  .view-transition-wrapper {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
  }

  .calendar-content > :global(*) {
    /* Ensure the children stretch to fill the absolute wrapper */
    flex: 1;
  }

  .slim-scroll::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  .slim-scroll::-webkit-scrollbar-track {
    background: transparent;
  }
  .slim-scroll::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 3px;
  }
  .slim-scroll::-webkit-scrollbar-thumb:hover {
    background: var(--text-tertiary);
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


</style>
