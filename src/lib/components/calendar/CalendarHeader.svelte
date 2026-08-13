<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import type { CalendarSource } from "$lib/types/calendar";

  let showJump = $state(false);
  let jumpMonth = $state("");
  let isPortrait = $derived(uiStore.windowOrientation === "left" || uiStore.windowOrientation === "right");
  let weekDays = $derived(calendarStore.generateWeekDays(calendarStore.currentDate));
  let title = $derived.by(() => {
    if (calendarStore.viewMode === "month") {
      return calendarStore.currentDate.toLocaleDateString("en-US", { month: "long", year: "numeric" });
    }
    const first = weekDays[0]?.date;
    const last = weekDays[6]?.date;
    if (!first || !last) return "Week";
    const firstText = first.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    const lastText = last.toLocaleDateString("en-US", {
      month: first.getMonth() === last.getMonth() ? undefined : "short",
      day: "numeric",
      year: first.getFullYear() === last.getFullYear() ? undefined : "numeric",
    });
    return `${firstText} – ${lastText}, ${last.getFullYear()}`;
  });

  const sources: { id: CalendarSource; label: string; short: string }[] = [
    { id: "tasks", label: "Tasks", short: "T" },
    { id: "local", label: "Events", short: "E" },
    { id: "google", label: "Google", short: "G" },
    { id: "time", label: "Actual time", short: "A" },
  ];

  function navigate(direction: -1 | 1) {
    const next = new Date(calendarStore.currentDate);
    if (calendarStore.viewMode === "month") next.setMonth(next.getMonth() + direction);
    else next.setDate(next.getDate() + direction * 7);
    calendarStore.setCurrentDate(next);
  }

  function openJump() {
    jumpMonth = `${calendarStore.currentDate.getFullYear()}-${String(calendarStore.currentDate.getMonth() + 1).padStart(2, "0")}`;
    showJump = true;
  }

  function applyJump() {
    const [year, month] = jumpMonth.split("-").map(Number);
    if (year && month) calendarStore.setCurrentDate(new Date(year, month - 1, 1));
    showJump = false;
  }
</script>

<header class="calendar-toolbar">
  <div class="primary-row">
    <button type="button" class="back-btn" aria-label="Back to projects" onclick={() => uiStore.closeCalendarView()}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6" /></svg>
      {#if !isPortrait}<span>Calendar</span>{/if}
    </button>

    <div class="date-navigation">
      <button type="button" class="icon-btn" aria-label="Previous period" onclick={() => navigate(-1)}>‹</button>
      <button type="button" class="date-title" onclick={openJump} aria-haspopup="dialog" aria-expanded={showJump}>{title}<span aria-hidden="true">⌄</span></button>
      <button type="button" class="icon-btn" aria-label="Next period" onclick={() => navigate(1)}>›</button>
      <button type="button" class="today-btn" onclick={() => calendarStore.setCurrentDate(new Date())}>Today</button>
      {#if showJump}
        <button type="button" class="backdrop" aria-label="Close date jump" onclick={() => (showJump = false)}></button>
        <div class="jump-popover" role="dialog" aria-label="Jump to month">
          <label><span>Month</span><input class="input" type="month" bind:value={jumpMonth} /></label>
          <button type="button" onclick={applyJump}>Go</button>
        </div>
      {/if}
    </div>

    <div class="toolbar-actions">
      <div class="view-toggle" aria-label="Calendar view">
        <button type="button" class:active={calendarStore.viewMode === "month"} aria-pressed={calendarStore.viewMode === "month"} onclick={() => calendarStore.setViewMode("month")}>{isPortrait ? "M" : "Month"}</button>
        <button type="button" class:active={calendarStore.viewMode === "week"} aria-pressed={calendarStore.viewMode === "week"} onclick={() => calendarStore.setViewMode("week")}>{isPortrait ? "W" : "Week"}</button>
      </div>
      <button type="button" class="new-event" onclick={() => calendarStore.openNewEvent()}>+ {isPortrait ? "" : "Event"}</button>
    </div>
  </div>

  <div class="source-row">
    <span class="source-caption">Show</span>
    {#each sources as source}
      <button
        type="button"
        class={`source-filter ${source.id}`}
        class:active={calendarStore.filters[source.id]}
        aria-pressed={calendarStore.filters[source.id]}
        onclick={() => calendarStore.toggleSource(source.id)}
        title={source.label}
      ><span class="source-shape"></span>{isPortrait ? source.short : source.label}</button>
    {/each}
    {#if calendarStore.googleIsStale}
      <span class="stale-status" title={calendarStore.googleError ?? "Showing cached Google events"}>Google offline · cached</span>
    {/if}
  </div>
</header>

<style>
  .calendar-toolbar { position:relative; flex:0 0 auto; border-bottom:1px solid var(--border); background:var(--bg-secondary); z-index:30; }
  .primary-row { min-height:56px; display:flex; align-items:center; gap:var(--spacing-md); padding:var(--spacing-sm) var(--spacing-md); }
  button { font:inherit; }
  .back-btn, .icon-btn, .today-btn, .date-title, .new-event { cursor:pointer; }
  .back-btn { display:flex; align-items:center; gap:7px; border:0; background:transparent; color:var(--text-secondary); padding:7px 9px; border-radius:var(--radius-md); font-size:13px; font-weight:600; }
  .back-btn:hover, .icon-btn:hover, .date-title:hover { background:var(--bg-hover); color:var(--text-primary); }
  .date-navigation { position:relative; display:flex; align-items:center; gap:4px; min-width:0; }
  .icon-btn { width:32px; height:32px; border:0; border-radius:50%; background:transparent; color:var(--text-secondary); font-size:24px; line-height:1; }
  .date-title { min-width:190px; max-width:260px; border:0; background:transparent; color:var(--text-primary); padding:7px 9px; border-radius:var(--radius-md); font-size:15px; font-weight:650; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
  .date-title span { margin-left:5px; color:var(--text-tertiary); }
  .today-btn { padding:6px 10px; border:1px solid var(--border); border-radius:var(--radius-md); background:var(--bg-primary); color:var(--text-secondary); font-size:12px; font-weight:600; }
  .today-btn:hover { border-color:var(--accent); color:var(--accent); }
  .toolbar-actions { display:flex; align-items:center; gap:var(--spacing-sm); margin-left:auto; }
  .view-toggle { display:grid; grid-template-columns:1fr 1fr; padding:3px; border:1px solid var(--border); border-radius:var(--radius-md); background:var(--bg-primary); }
  .view-toggle button { min-width:58px; border:0; border-radius:calc(var(--radius-md) - 3px); padding:5px 9px; background:transparent; color:var(--text-secondary); cursor:pointer; font-size:11px; font-weight:650; }
  .view-toggle button.active { background:var(--bg-hover); color:var(--text-primary); box-shadow:0 1px 2px rgba(0,0,0,.12); }
  .new-event { padding:7px 11px; border:1px solid var(--accent); border-radius:var(--radius-md); background:var(--accent); color:var(--accent-contrast); font-size:12px; font-weight:700; }
  .source-row { min-height:38px; display:flex; align-items:center; gap:6px; padding:0 var(--spacing-md) var(--spacing-sm); overflow-x:auto; }
  .source-caption { margin-right:2px; color:var(--text-tertiary); font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:.06em; }
  .source-filter { display:flex; align-items:center; gap:6px; white-space:nowrap; padding:4px 8px; border:1px solid var(--border-light); border-radius:999px; background:transparent; color:var(--text-tertiary); cursor:pointer; font-size:10px; font-weight:650; opacity:.65; }
  .source-filter.active { background:var(--bg-primary); color:var(--text-secondary); border-color:var(--border); opacity:1; }
  .source-shape { width:7px; height:7px; border-radius:50%; background:var(--text-tertiary); }
  .tasks .source-shape { border-radius:2px; background:var(--accent); }
  .local .source-shape { background:var(--success); }
  .google .source-shape { background:#4285f4; outline:1px solid #4285f4; outline-offset:1px; }
  .time .source-shape { border-radius:1px; transform:rotate(45deg); }
  .stale-status { margin-left:auto; color:var(--warning); font-size:10px; white-space:nowrap; }
  .backdrop { position:fixed; inset:0; border:0; background:transparent; z-index:40; }
  .jump-popover { position:absolute; left:70px; top:calc(100% + 8px); display:flex; align-items:flex-end; gap:8px; padding:var(--spacing-md); border:1px solid var(--border); border-radius:var(--radius-lg); background:var(--bg-secondary); box-shadow:0 12px 32px rgba(0,0,0,.22); z-index:41; }
  .jump-popover label { display:flex; flex-direction:column; gap:5px; color:var(--text-tertiary); font-size:10px; text-transform:uppercase; }
  .jump-popover button { height:34px; border:1px solid var(--accent); border-radius:var(--radius-md); background:var(--accent); color:var(--accent-contrast); font-weight:600; cursor:pointer; }
  @media (max-width:720px) { .primary-row { gap:5px; padding-inline:var(--spacing-sm); } .date-title { min-width:130px; font-size:13px; } .today-btn { display:none; } .view-toggle button { min-width:34px; } .source-row { padding-inline:var(--spacing-sm); } }
  :global(body.compact-mode) .primary-row { min-height:48px; }
  @media (prefers-reduced-motion:reduce) { * { transition:none !important; } }
</style>
