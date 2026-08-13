<script lang="ts">
  import { onMount, tick } from "svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import {
    dateToKey,
    minutesToTime,
    positionTimedItems,
  } from "$lib/components/calendar/calendar-utils";
  import type { CalendarItem } from "$lib/types/calendar";
  import { shouldSelectWeekGridTarget } from "$lib/components/calendar/calendar-interaction-policy";

  const pixelsPerMinute = 0.8;
  const dayHeight = 24 * 60 * pixelsPerMinute;
  const hourHeight = 60 * pixelsPerMinute;
  let scrollViewport: HTMLDivElement;
  let weekDays = $derived(calendarStore.generateWeekDays(calendarStore.currentDate));
  let isPortrait = $derived(uiStore.windowOrientation === "left" || uiStore.windowOrientation === "right");
  let focusDate = $derived(calendarStore.selectedDate ?? calendarStore.currentDate);
  let shownDays = $derived(isPortrait
    ? weekDays.filter((day) => dateToKey(day.date) === dateToKey(focusDate)).slice(0, 1)
    : weekDays);
  let hours = Array.from({ length: 24 }, (_, index) => index);
  let now = $state(new Date());
  let resizeState = $state<{ item: CalendarItem; startY: number; startDuration: number } | null>(null);

  onMount(() => {
    const timer = window.setInterval(() => (now = new Date()), 60_000);
    void tick().then(() => {
      const initialHour = dateToKey(now) === dateToKey(focusDate) ? Math.max(0, now.getHours() - 2) : 7;
      scrollViewport?.scrollTo({ top: initialHour * hourHeight, behavior: "instant" });
    });
    return () => window.clearInterval(timer);
  });

  function itemsForDay(date: Date) {
    return calendarStore.getItemsForDate(dateToKey(date));
  }

  function timedForDay(date: Date) {
    return positionTimedItems(itemsForDay(date).filter((item) => !item.isAllDay), pixelsPerMinute);
  }

  function allDayForDay(date: Date) {
    return itemsForDay(date).filter((item) => item.isAllDay);
  }

  function formatHour(hour: number): string {
    if (hour === 0) return "12 AM";
    if (hour === 12) return "12 PM";
    return `${hour % 12} ${hour < 12 ? "AM" : "PM"}`;
  }

  function nowTop(): number {
    return (now.getHours() * 60 + now.getMinutes()) * pixelsPerMinute;
  }

  function setFocusedDay(date: Date) {
    calendarStore.setSelectedDate(date);
    calendarStore.setCurrentDate(date);
  }

  function selectGridDay(event: MouseEvent, date: Date) {
    if (!shouldSelectWeekGridTarget(event.target as Element | null)) return;
    calendarStore.setSelectedDate(date);
  }

  function dragStart(event: DragEvent, item: CalendarItem) {
    if (item.readOnly || item.kind === "time_entry" || (item.kind === "local_event" && !!item.event.recurrence_rule)) return;
    event.dataTransfer?.setData("calendar-item-key", item.key);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  async function dropOnGrid(event: DragEvent, date: Date) {
    event.preventDefault();
    const key = event.dataTransfer?.getData("calendar-item-key");
    const item = calendarStore.allItems.find((candidate) => candidate.key === key);
    if (!item) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const minutes = Math.max(0, Math.min(23 * 60 + 45, Math.round(((event.clientY - rect.top) / pixelsPerMinute) / 15) * 15));
    await calendarStore.rescheduleItem(item, dateToKey(date), minutesToTime(minutes));
  }

  function resizeStart(event: PointerEvent, item: CalendarItem) {
    event.stopPropagation();
    const start = item.startAt && item.endAt
      ? Math.max(15, Math.round((item.endAt - item.startAt) / 60))
      : item.kind === "task" ? item.task.planned_duration_minutes ?? 30 : 30;
    resizeState = { item, startY: event.clientY, startDuration: start };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  async function resizeEnd(event: PointerEvent) {
    if (!resizeState) return;
    const duration = resizeState.startDuration + (event.clientY - resizeState.startY) / pixelsPerMinute;
    const item = resizeState.item;
    resizeState = null;
    await calendarStore.resizeItem(item, duration);
  }

  function itemTime(item: CalendarItem): string {
    if (!item.startTime) return "";
    return new Date(`2000-01-01T${item.startTime}:00`).toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
  }
</script>

<div class="week-shell" class:portrait={isPortrait}>
  {#if isPortrait}
    <div class="day-strip" aria-label="Days in week">
      {#each weekDays as day}
        <button type="button" class:active={dateToKey(day.date) === dateToKey(focusDate)} class:today={dateToKey(day.date) === dateToKey(new Date())} onclick={() => setFocusedDay(day.date)}>
          <span>{day.dayName}</span><strong>{day.date.getDate()}</strong>
        </button>
      {/each}
    </div>
  {/if}

  <div class="all-day-header">
    <div class="all-day-label">All day</div>
    {#each shownDays as day}
      <div class="all-day-column">
        {#if !isPortrait}<button type="button" class:today={dateToKey(day.date) === dateToKey(new Date())} onclick={() => calendarStore.setSelectedDate(day.date)}><span>{day.dayName}</span><strong>{day.date.getDate()}</strong></button>{/if}
        <div class="all-day-items">
          {#each allDayForDay(day.date).slice(0, 3) as item (item.key)}
            <button type="button" class={`all-day-item ${item.source}`} style={`--item-color:${item.color}`} onclick={() => calendarStore.selectItem(item)} draggable={!item.readOnly && item.kind !== "time_entry" && !(item.kind === "local_event" && !!item.event.recurrence_rule)} ondragstart={(event) => dragStart(event, item)}>{item.title}</button>
          {/each}
          {#if allDayForDay(day.date).length > 3}<small>+{allDayForDay(day.date).length - 3}</small>{/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="week-scroll" bind:this={scrollViewport}>
    <div class="time-gutter" style={`height:${dayHeight}px`}>
      {#each hours as hour}<span style={`top:${hour * hourHeight}px`}>{formatHour(hour)}</span>{/each}
    </div>
    {#each shownDays as day}
      <div
        class="day-lane"
        style={`height:${dayHeight}px`}
        role="gridcell"
        tabindex="0"
        aria-label={`Schedule for ${day.date.toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" })}`}
        onclick={(event) => selectGridDay(event, day.date)}
        onkeydown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            calendarStore.setSelectedDate(day.date);
          }
        }}
        ondragover={(event) => event.preventDefault()}
        ondrop={(event) => dropOnGrid(event, day.date)}
      >
        {#each hours as hour}<div class="hour-line" style={`top:${hour * hourHeight}px`}></div>{/each}
        {#if dateToKey(day.date) === dateToKey(now)}
          <div class="now-line" style={`top:${nowTop()}px`}><span></span></div>
        {/if}
        {#each timedForDay(day.date) as positioned (positioned.item.key)}
          {@const item = positioned.item}
          <button
            type="button"
            class={`timed-item ${item.source}`}
            class:completed={item.kind === "task" && item.task.completed}
            style={`--item-color:${item.color};top:${positioned.top}px;height:${positioned.height}px;left:calc(${positioned.column * (100 / positioned.columnCount)}% + 3px);width:calc(${100 / positioned.columnCount}% - 6px)`}
            draggable={!item.readOnly && item.kind !== "time_entry" && !(item.kind === "local_event" && !!item.event.recurrence_rule)}
            onclick={(event) => { event.stopPropagation(); calendarStore.selectItem(item); }}
            ondragstart={(event) => dragStart(event, item)}
          >
            <span class="timed-title">{item.title}</span>
            <span class="timed-meta">{itemTime(item)}{item.kind === "google_event" ? " · Google" : item.kind === "time_entry" ? " · Actual" : ""}</span>
            {#if !item.readOnly && item.kind !== "time_entry" && !(item.kind === "local_event" && !!item.event.recurrence_rule)}<span class="resize-handle" role="separator" aria-label={`Resize ${item.title}`} onpointerdown={(event) => resizeStart(event, item)} onpointerup={resizeEnd}></span>{/if}
          </button>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .week-shell { height:100%; min-height:0; display:flex; flex-direction:column; background:var(--bg-primary); }
  .all-day-header, .week-scroll { display:grid; grid-template-columns:54px repeat(7,minmax(92px,1fr)); }
  .all-day-header { flex:0 0 auto; min-height:68px; border-bottom:1px solid var(--border); background:var(--bg-secondary); }
  .all-day-label { display:flex; align-items:flex-end; justify-content:flex-end; padding:0 8px 7px 0; color:var(--text-tertiary); font-size:9px; font-weight:700; text-transform:uppercase; }
  .all-day-column { min-width:0; border-left:1px solid var(--border-light); padding:5px; }
  .all-day-column > button { width:100%; display:flex; align-items:center; justify-content:center; gap:6px; border:0; background:transparent; color:var(--text-secondary); cursor:pointer; }
  .all-day-column > button span { font-size:10px; text-transform:uppercase; }
  .all-day-column > button strong { width:24px; height:24px; display:grid; place-items:center; border-radius:50%; font-size:13px; }
  .all-day-column > button.today strong { background:var(--accent); color:var(--accent-contrast); }
  .all-day-items { display:flex; flex-direction:column; gap:2px; margin-top:3px; }
  .all-day-item { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; border:0; border-left:3px solid var(--item-color); border-radius:3px; background:color-mix(in srgb,var(--item-color) 10%,var(--bg-primary)); color:var(--text-primary); padding:2px 4px; text-align:left; font-size:9px; cursor:pointer; }
  .all-day-item.google { border-left-style:dashed; background:transparent; }
  .all-day-items small { color:var(--text-tertiary); font-size:8px; }
  .week-scroll { flex:1; min-height:0; overflow:auto; position:relative; align-items:start; }
  .time-gutter { position:relative; border-right:1px solid var(--border); background:var(--bg-secondary); }
  .time-gutter span { position:absolute; right:7px; transform:translateY(-50%); color:var(--text-tertiary); font-size:9px; font-variant-numeric:tabular-nums; }
  .day-lane { min-width:0; position:relative; border-right:1px solid var(--border-light); cursor:crosshair; }
  .hour-line { position:absolute; left:0; right:0; border-top:1px solid var(--border-light); pointer-events:none; }
  .now-line { position:absolute; left:-1px; right:0; height:1px; background:var(--danger); z-index:8; pointer-events:none; }
  .now-line span { position:absolute; left:-3px; top:-3px; width:7px; height:7px; border-radius:50%; background:var(--danger); }
  .timed-item { position:absolute; z-index:4; display:flex; flex-direction:column; align-items:flex-start; gap:1px; overflow:hidden; padding:4px 5px; border:0; border-left:3px solid var(--item-color); border-radius:4px; background:color-mix(in srgb,var(--item-color) 13%,var(--bg-primary)); color:var(--text-primary); text-align:left; cursor:pointer; box-shadow:0 1px 2px rgba(0,0,0,.12); }
  .timed-item:hover { z-index:6; background:color-mix(in srgb,var(--item-color) 20%,var(--bg-primary)); }
  .timed-item.google { border-left-style:dashed; background:color-mix(in srgb,var(--item-color) 6%,var(--bg-primary)); }
  .timed-item.time { opacity:.72; background:repeating-linear-gradient(135deg,color-mix(in srgb,var(--item-color) 12%,var(--bg-primary)) 0 6px,var(--bg-primary) 6px 12px); }
  .timed-item.completed { opacity:.55; }
  .timed-title { width:100%; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:10px; font-weight:650; }
  .completed .timed-title { text-decoration:line-through; }
  .timed-meta { width:100%; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:var(--text-secondary); font-size:8px; }
  .resize-handle { position:absolute; left:4px; right:4px; bottom:0; height:5px; cursor:ns-resize; border-bottom:2px solid color-mix(in srgb,var(--item-color) 55%,transparent); }
  .day-strip { display:grid; grid-template-columns:repeat(7,1fr); gap:2px; padding:5px; border-bottom:1px solid var(--border); background:var(--bg-secondary); }
  .day-strip button { display:flex; flex-direction:column; align-items:center; gap:2px; border:0; border-radius:var(--radius-md); padding:5px 2px; background:transparent; color:var(--text-secondary); cursor:pointer; }
  .day-strip button span { font-size:8px; text-transform:uppercase; }
  .day-strip button strong { width:23px; height:23px; display:grid; place-items:center; border-radius:50%; font-size:11px; }
  .day-strip button.active { background:var(--bg-hover); color:var(--text-primary); }
  .day-strip button.today strong { background:var(--accent); color:var(--accent-contrast); }
  .portrait .all-day-header, .portrait .week-scroll { grid-template-columns:48px minmax(0,1fr); }
  .portrait .all-day-header { min-height:44px; }
  :global(body.compact-mode) .timed-item { padding:2px 4px; }
  @media (prefers-reduced-motion:reduce) { * { scroll-behavior:auto !important; transition:none !important; } }
</style>
