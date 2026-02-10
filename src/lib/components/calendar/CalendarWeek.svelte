<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { calendarStore } from '$lib/stores/calendar.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import type { TimeEntryWithTask } from '$lib/types/calendar';
  import TimeDisplay from '$lib/components/common/TimeDisplay.svelte';

  interface PositionedEntry {
    entry: TimeEntryWithTask;
    top: number;
    height: number;
    leftPercent: number;
    widthPercent: number;
  }

  interface WorkingEntry {
    entry: TimeEntryWithTask;
    startMinutes: number;
    endMinutes: number;
    lane: number;
  }

  const hours = Array.from({ length: 24 }, (_, i) => i);
  let weekContainer = $state<HTMLDivElement | null>(null);
  let isPortrait = $derived(uiStore.windowOrientation === 'left' || uiStore.windowOrientation === 'right');
  let pxPerMinute = $derived(isPortrait ? 0.8 : 1);
  let hourHeight = $derived(60 * pxPerMinute);
  let timelineHeight = $derived(1440 * pxPerMinute);

  let weekStart = $derived(calendarStore.getWeekStart(calendarStore.currentDate));

  let weekDays = $derived(calendarStore.generateWeekDays(weekStart));

  function getEntriesForDay(date: Date): TimeEntryWithTask[] {
    const dateStr = calendarStore.dateToString(date);
    return [...(calendarStore.timeEntriesByDate.get(dateStr) || [])].sort(
      (a, b) => a.started_at - b.started_at
    );
  }

  function getMinutesFromTimestamp(timestamp: number): number {
    const date = new Date(timestamp * 1000);
    return (date.getHours() * 60) + date.getMinutes();
  }

  function getLaidOutEntries(date: Date): PositionedEntry[] {
    const entries = getEntriesForDay(date);
    if (entries.length === 0) {
      return [];
    }

    const workingEntries: WorkingEntry[] = entries.map((entry) => {
      const startMinutes = Math.max(0, Math.min(1439, getMinutesFromTimestamp(entry.started_at)));
      const durationMinutes = Math.max(1, Math.round(entry.duration_seconds / 60));
      const endMinutes = Math.min(1440, startMinutes + durationMinutes);
      return {
        entry,
        startMinutes,
        endMinutes,
        lane: 0,
      };
    });

    const clusters: WorkingEntry[][] = [];
    let currentCluster: WorkingEntry[] = [];
    let clusterEnd = -1;

    for (const entry of workingEntries) {
      if (currentCluster.length === 0 || entry.startMinutes < clusterEnd) {
        currentCluster.push(entry);
        clusterEnd = Math.max(clusterEnd, entry.endMinutes);
      } else {
        clusters.push(currentCluster);
        currentCluster = [entry];
        clusterEnd = entry.endMinutes;
      }
    }

    if (currentCluster.length > 0) {
      clusters.push(currentCluster);
    }

    const laidOut: PositionedEntry[] = [];

    for (const cluster of clusters) {
      const laneEnds: number[] = [];
      for (const entry of cluster) {
        let lane = laneEnds.findIndex((endMinutes) => endMinutes <= entry.startMinutes);
        if (lane === -1) {
          lane = laneEnds.length;
          laneEnds.push(entry.endMinutes);
        } else {
          laneEnds[lane] = entry.endMinutes;
        }
        entry.lane = lane;
      }

      const laneCount = Math.max(1, laneEnds.length);
      for (const entry of cluster) {
        laidOut.push({
          entry: entry.entry,
          top: entry.startMinutes * pxPerMinute,
          height: Math.max((entry.endMinutes - entry.startMinutes) * pxPerMinute, 20),
          leftPercent: (entry.lane / laneCount) * 100,
          widthPercent: 100 / laneCount,
        });
      }
    }

    return laidOut;
  }

  function isToday(date: Date): boolean {
    const today = new Date();
    return date.toDateString() === today.toDateString();
  }

  function getNowMarkerTop(date: Date): number | null {
    if (!isToday(date)) {
      return null;
    }
    const now = new Date();
    const nowMinutes = (now.getHours() * 60) + now.getMinutes();
    return nowMinutes * pxPerMinute;
  }

  async function scrollToNow() {
    await tick();
    if (!weekContainer) {
      return;
    }

    const today = weekDays.find((day) => isToday(day.date));
    if (!today) {
      return;
    }

    const nowTop = getNowMarkerTop(today.date);
    if (nowTop === null) {
      return;
    }

    weekContainer.scrollTop = Math.max(0, nowTop - (hourHeight * 2));
  }

  onMount(() => {
    void scrollToNow();
  });

  $effect(() => {
    weekStart;
    pxPerMinute;
    void scrollToNow();
  });
</script>

<div class="calendar-week" class:portrait={isPortrait} bind:this={weekContainer}>
  <div class="time-axis">
    <div class="corner-cell"></div>
    {#each hours as hour}
      <div class="hour-label" class:minor={hour % 6 !== 0} style="height: {hourHeight}px">
        {#if hour % 6 === 0}
          {hour}:00
        {/if}
      </div>
    {/each}
  </div>

  {#each weekDays as day}
    <div class="day-column" class:today={isToday(day.date)}>
      <div class="day-header">
        <span class="day-name">{day.dayName}</span>
        <span class="day-number">{day.date.getDate()}</span>
      </div>

      <div class="day-timeline" style="height: {timelineHeight}px">
        {#each getLaidOutEntries(day.date) as pos (pos.entry.id)}
          <button
            class="time-block"
            class:selected={uiStore.calendarSelectedEntry?.id === pos.entry.id}
            style="top: {pos.top}px; height: {pos.height}px; left: calc({pos.leftPercent}% + 4px); width: calc({pos.widthPercent}% - 8px); background-color: {pos.entry.project_color || 'var(--accent)'}"
            onclick={() => uiStore.selectCalendarEntry(pos.entry)}
          >
            <span class="block-task">{pos.entry.task_title}</span>
            <span class="block-duration">
              <TimeDisplay seconds={pos.entry.duration_seconds} format="short" />
            </span>
          </button>
        {/each}

        {#if getNowMarkerTop(day.date) !== null}
          <div class="now-marker" style="top: {getNowMarkerTop(day.date)}px">
            <span class="now-label">Now</span>
          </div>
        {/if}

        <div class="hour-grid">
          {#each hours as hour}
            <div class="hour-line" class:minor={hour % 6 !== 0} style="height: {hourHeight}px"></div>
          {/each}
        </div>
      </div>
    </div>
  {/each}
</div>

<style>
  .calendar-week {
    display: grid;
    grid-template-columns: 50px repeat(7, 1fr);
    height: 100%;
    overflow: auto;
  }

  .calendar-week.portrait {
    grid-template-columns: 40px repeat(7, 1fr);
  }

  .time-axis {
    position: sticky;
    left: 0;
    top: 0;
    z-index: 20;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
  }

  .corner-cell {
    height: 50px;
    border-bottom: 1px solid var(--border);
  }

  .hour-label {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    padding: 4px 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    border-bottom: 1px solid transparent;
  }

  .hour-label.minor {
    color: var(--text-tertiary);
  }

  .portrait .hour-label {
    font-size: 10px;
  }

  .day-column {
    border-left: 1px solid var(--border-light);
    position: relative;
  }

  .day-column.today {
    background: color-mix(in srgb, var(--accent) 5%, transparent);
  }

  .day-header {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 50px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .portrait .day-header {
    height: 40px;
    font-size: 11px;
  }

  .day-name {
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .day-number {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .day-column.today .day-number {
    background: var(--accent);
    color: white;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .day-timeline {
    position: relative;
  }

  .hour-grid {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .hour-line {
    border-bottom: 1px solid var(--border-light);
  }

  .hour-line.minor {
    border-bottom: 1px dashed var(--border-light);
    opacity: 0.5;
  }

  .time-block {
    position: absolute;
    border-radius: var(--radius-sm);
    padding: 4px 6px;
    font-size: 11px;
    color: white;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.15s;
    border: none;
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 2px;
    box-shadow: var(--shadow-sm);
  }

  .time-block:hover {
    transform: scale(1.02);
    z-index: 5;
    box-shadow: var(--shadow-md);
  }

  .time-block.selected {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    z-index: 10;
  }

  .now-marker {
    position: absolute;
    left: 0;
    right: 0;
    border-top: 2px solid var(--danger);
    z-index: 6;
    pointer-events: none;
  }

  .now-label {
    position: absolute;
    top: -8px;
    right: 4px;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: white;
    background: var(--danger);
    border-radius: 10px;
    padding: 1px 6px;
    text-transform: uppercase;
  }

  .block-task {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .block-duration {
    font-size: 10px;
    opacity: 0.9;
    font-family: var(--font-mono);
  }
</style>
