<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { navigateCalendarPeriod } from "$lib/components/calendar/calendar-utils";
  import type { CalendarSource } from "$lib/types/calendar";

  let showJump = $state(false);
  let pickerYear = $state(new Date().getFullYear());
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
  const months = Array.from({ length: 12 }, (_, index) => ({
    index,
    label: new Date(2000, index, 1).toLocaleDateString("en-US", { month: "short" }),
    fullLabel: new Date(2000, index, 1).toLocaleDateString("en-US", { month: "long" }),
  }));

  function navigate(direction: -1 | 1) {
    calendarStore.setCurrentDate(
      navigateCalendarPeriod(calendarStore.currentDate, calendarStore.viewMode, direction),
    );
  }

  function openJump() {
    pickerYear = calendarStore.currentDate.getFullYear();
    showJump = true;
  }

  function selectMonth(month: number) {
    calendarStore.setCurrentDate(new Date(pickerYear, month, 1));
    showJump = false;
  }
</script>

<header class="calendar-toolbar">
  <div class="primary-row">
    <div class="date-navigation">
      <button type="button" class="icon-btn" aria-label="Previous period" onclick={() => navigate(-1)}>‹</button>
      <button type="button" class="date-title" onclick={openJump} aria-haspopup="dialog" aria-expanded={showJump}>{title}<span aria-hidden="true">⌄</span></button>
      <button type="button" class="icon-btn" aria-label="Next period" onclick={() => navigate(1)}>›</button>
      <button type="button" class="today-btn" onclick={() => calendarStore.setCurrentDate(new Date())}>Today</button>
      {#if showJump}
        <button type="button" class="backdrop" aria-label="Close date jump" onclick={() => (showJump = false)}></button>
        <div class="jump-popover" role="dialog" aria-label="Jump to month">
          <div class="picker-year">
            <button type="button" class="year-arrow" aria-label="Previous year" onclick={() => (pickerYear -= 1)}>‹</button>
            <strong>{pickerYear}</strong>
            <button type="button" class="year-arrow" aria-label="Next year" onclick={() => (pickerYear += 1)}>›</button>
          </div>
          <div class="month-grid" aria-label={`Months in ${pickerYear}`}>
            {#each months as month}
              <button
                type="button"
                class="month-option"
                class:active={pickerYear === calendarStore.currentDate.getFullYear() && month.index === calendarStore.currentDate.getMonth()}
                aria-label={`${month.fullLabel} ${pickerYear}`}
                onclick={() => selectMonth(month.index)}
              >{month.label}</button>
            {/each}
          </div>
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
  .icon-btn, .today-btn, .date-title, .new-event { cursor:pointer; }
  .icon-btn:hover, .date-title:hover { background:var(--bg-hover); color:var(--text-primary); }
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
  .jump-popover { position:absolute; left:34px; top:calc(100% + 8px); width:236px; padding:var(--spacing-md); border:1px solid var(--border); border-radius:var(--radius-lg); background:var(--bg-secondary); box-shadow:0 12px 32px rgba(0,0,0,.22); z-index:41; }
  .picker-year { display:grid; grid-template-columns:34px 1fr 34px; align-items:center; margin-bottom:var(--spacing-sm); }
  .picker-year strong { color:var(--text-primary); text-align:center; font-size:14px; }
  .year-arrow { width:32px; height:32px; border:0; border-radius:50%; background:transparent; color:var(--text-secondary); cursor:pointer; font-size:21px; }
  .year-arrow:hover { background:var(--bg-hover); color:var(--text-primary); }
  .month-grid { display:grid; grid-template-columns:repeat(3,1fr); gap:5px; }
  .month-option { height:34px; border:1px solid transparent; border-radius:var(--radius-md); background:var(--bg-primary); color:var(--text-secondary); cursor:pointer; font-size:11px; font-weight:600; }
  .month-option:hover { border-color:var(--border); background:var(--bg-hover); color:var(--text-primary); }
  .month-option.active { border-color:var(--accent); background:color-mix(in srgb,var(--accent) 12%,var(--bg-primary)); color:var(--accent); }
  @media (max-width:720px) { .primary-row { gap:5px; padding-inline:var(--spacing-sm); } .date-title { min-width:130px; font-size:13px; } .today-btn { display:none; } .view-toggle button { min-width:34px; } .source-row { padding-inline:var(--spacing-sm); } }
  :global(body.compact-mode) .primary-row { min-height:48px; }
  @media (prefers-reduced-motion:reduce) { * { transition:none !important; } }
</style>
