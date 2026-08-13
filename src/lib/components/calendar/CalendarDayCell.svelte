<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import type { CalendarDay, CalendarItem } from "$lib/types/calendar";

  let { day } = $props<{ day: CalendarDay }>();
  let isDragOver = $state(false);
  let visibleItems = $derived(day.items.slice(0, 3));
  let overflowCount = $derived(Math.max(0, day.items.length - visibleItems.length));

  function sourceSymbol(item: CalendarItem): string {
    if (item.kind === "task") return item.task.completed ? "✓" : "□";
    if (item.kind === "google_event") return "G";
    if (item.kind === "time_entry") return "◷";
    return item.event.recurrence_rule ? "↻" : "●";
  }

  function selectDay() {
    calendarStore.setSelectedDate(day.date);
  }

  function selectItem(event: MouseEvent, item: CalendarItem) {
    event.stopPropagation();
    calendarStore.selectItem(item);
  }

  function dragStart(event: DragEvent, item: CalendarItem) {
    if (item.readOnly || item.kind === "time_entry" || (item.kind === "local_event" && !!item.event.recurrence_rule)) return;
    event.dataTransfer?.setData("calendar-item-key", item.key);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  async function drop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
    const key = event.dataTransfer?.getData("calendar-item-key");
    const item = calendarStore.allItems.find((candidate) => candidate.key === key);
    if (item) await calendarStore.rescheduleItem(item, day.dateKey, null);
  }
</script>

<div
  class="day-cell"
  class:today={day.isToday}
  class:selected={day.isSelected}
  class:other-month={!day.isCurrentMonth}
  class:weekend={day.date.getDay() === 0 || day.date.getDay() === 6}
  class:drag-over={isDragOver}
  role="button"
  tabindex="0"
  aria-label={day.date.toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric", year: "numeric" })}
  onclick={selectDay}
  onkeydown={(event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      selectDay();
    }
  }}
  ondragover={(event) => { event.preventDefault(); isDragOver = day.isCurrentMonth; }}
  ondragleave={(event) => { if (!(event.currentTarget as HTMLElement).contains(event.relatedTarget as Node)) isDragOver = false; }}
  ondrop={drop}
>
  <div class="day-heading">
    <span class="day-number">{day.date.getDate()}</span>
  </div>

  <div class="item-stack">
    {#each visibleItems as item (item.key)}
      <button
        type="button"
        class={`calendar-chip ${item.source}`}
        class:completed={item.kind === "task" && item.task.completed}
        draggable={!item.readOnly && item.kind !== "time_entry" && !(item.kind === "local_event" && !!item.event.recurrence_rule)}
        style={`--item-color:${item.color}`}
        onclick={(event) => selectItem(event, item)}
        ondragstart={(event) => dragStart(event, item)}
        title={`${item.title}${item.startTime ? ` · ${item.startTime}` : ""}`}
      >
        <span class="item-symbol" aria-hidden="true">{sourceSymbol(item)}</span>
        <span class="item-title">{item.title}</span>
        {#if item.startTime}<span class="item-time">{item.startTime}</span>{/if}
      </button>
    {/each}
    {#if overflowCount > 0}
      <button type="button" class="more-items" onclick={(event) => { event.stopPropagation(); selectDay(); }}>+{overflowCount} more</button>
    {/if}
  </div>

  {#if day.items.length > 0}
    <div class="compact-dots" aria-hidden="true">
      {#each day.items.slice(0, 4) as item (item.key)}<span style={`--item-color:${item.color}`}></span>{/each}
      {#if day.items.length > 4}<small>+{day.items.length - 4}</small>{/if}
    </div>
  {/if}
</div>

<style>
  .day-cell { min-width:0; min-height:104px; display:flex; flex-direction:column; gap:4px; padding:6px; border-right:1px solid var(--border-light); border-bottom:1px solid var(--border-light); background:var(--bg-primary); cursor:pointer; transition:background .12s, box-shadow .12s; }
  .day-cell:hover { background:var(--bg-hover); }
  .day-cell.weekend:not(.today) { background:color-mix(in srgb,var(--text-tertiary) 3%,var(--bg-primary)); }
  .day-cell.other-month { color:var(--text-tertiary); background:var(--bg-secondary); opacity:.62; }
  .day-cell.today { background:color-mix(in srgb,var(--accent) 5%,var(--bg-primary)); }
  .day-cell.selected { box-shadow:inset 0 0 0 2px var(--accent); z-index:2; }
  .day-cell.drag-over { box-shadow:inset 0 0 0 2px var(--accent); background:color-mix(in srgb,var(--accent) 12%,var(--bg-primary)); }
  .day-heading { min-height:24px; display:flex; align-items:center; justify-content:space-between; }
  .day-number { width:24px; height:24px; display:grid; place-items:center; border-radius:50%; color:var(--text-secondary); font-size:12px; font-weight:650; }
  .today .day-number { background:var(--accent); color:var(--accent-contrast); }
  .item-stack { min-width:0; display:flex; flex-direction:column; gap:3px; }
  .calendar-chip { min-width:0; height:20px; display:flex; align-items:center; gap:4px; padding:2px 5px; border:0; border-left:3px solid var(--item-color); border-radius:4px; background:color-mix(in srgb,var(--item-color) 9%,var(--bg-primary)); color:var(--text-primary); text-align:left; cursor:pointer; }
  .calendar-chip:hover { background:color-mix(in srgb,var(--item-color) 17%,var(--bg-primary)); }
  .calendar-chip.google { border-left-style:dashed; background:transparent; box-shadow:inset 0 0 0 1px color-mix(in srgb,var(--item-color) 28%,transparent); }
  .calendar-chip.time { opacity:.72; background:repeating-linear-gradient(135deg,color-mix(in srgb,var(--item-color) 10%,transparent) 0 4px,transparent 4px 8px); }
  .calendar-chip.completed { opacity:.55; }
  .item-symbol { width:11px; flex:0 0 auto; color:var(--item-color); font-size:9px; text-align:center; }
  .item-title { min-width:0; flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:10px; font-weight:550; }
  .completed .item-title { text-decoration:line-through; }
  .item-time { flex:0 0 auto; color:var(--text-tertiary); font-size:8px; font-variant-numeric:tabular-nums; }
  .more-items { align-self:flex-start; border:0; background:transparent; color:var(--text-secondary); padding:1px 4px; font-size:9px; font-weight:650; cursor:pointer; }
  .compact-dots { display:none; align-items:center; gap:3px; margin:auto 2px 1px; }
  .compact-dots span { width:6px; height:6px; border-radius:50%; background:var(--item-color); }
  .compact-dots small { color:var(--text-tertiary); font-size:8px; }
  @media (max-width:760px) {
    .day-cell { min-height:76px; padding:4px; }
    .item-stack { display:none; }
    .compact-dots { display:flex; }
  }
  :global(body.compact-mode) .day-cell { min-height:84px; padding:4px; }
  :global(body.compact-mode) .calendar-chip { height:17px; }
</style>
