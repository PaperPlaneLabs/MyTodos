<script lang="ts">
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import type { CalendarItem } from "$lib/types/calendar";
  import EventEditor from "$lib/components/calendar/EventEditor.svelte";

  let editingTime = $state(false);
  let timeMinutes = $state(0);
  let timeNote = $state("");
  let error = $state<string | null>(null);

  let selectedDateKey = $derived(
    calendarStore.selectedDate ? calendarStore.dateToString(calendarStore.selectedDate) : null,
  );
  let dateItems = $derived(selectedDateKey ? calendarStore.getItemsForDate(selectedDateKey) : []);

  function sourceLabel(item: CalendarItem): string {
    if (item.kind === "task") return "Task";
    if (item.kind === "local_event") return item.event.recurrence_rule ? "Recurring event" : "Event";
    if (item.kind === "google_event") return "Google Calendar";
    return "Tracked time";
  }

  function formatTime(item: CalendarItem): string {
    if (item.isAllDay) return "All day";
    const format = (value: string | null) => value
      ? new Date(`2000-01-01T${value}:00`).toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" })
      : "";
    return `${format(item.startTime)}${item.endTime ? ` – ${format(item.endTime)}` : ""}`;
  }

  async function deleteSelected(item: CalendarItem) {
    if (item.kind === "local_event") {
      if (confirm(item.event.recurrence_rule ? "Delete this entire recurring series?" : "Delete this event?")) {
        await calendarStore.deleteEvent(item.event.series_id ?? item.event.id);
      }
    } else if (item.kind === "time_entry") {
      if (confirm("Delete this tracked-time entry? This updates task and project totals.")) {
        await calendarStore.deleteTimeEntry(item.entry.id);
      }
    }
  }

  function startEditingTime(item: Extract<CalendarItem, { kind: "time_entry" }>) {
    timeMinutes = Math.max(1, Math.round(item.entry.duration_seconds / 60));
    timeNote = item.entry.note ?? "";
    editingTime = true;
  }

  async function saveTime(item: Extract<CalendarItem, { kind: "time_entry" }>) {
    try {
      await calendarStore.updateTimeEntry(item.entry.id, Math.max(1, timeMinutes) * 60, timeNote.trim() || null);
      editingTime = false;
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    }
  }
</script>

<aside class="inspector" aria-label="Calendar details">
  {#if calendarStore.editorDraft}
    {#key `${calendarStore.editorDraft.id}:${calendarStore.editorDraft.startDate}`}
      <EventEditor draft={calendarStore.editorDraft} />
    {/key}
  {:else}
    <div class="inspector-header">
      <div>
        <span class="eyebrow">Calendar details</span>
        <h3>
          {#if calendarStore.selectedItem}{calendarStore.selectedItem.title}
          {:else if calendarStore.selectedDate}
            {calendarStore.selectedDate.toLocaleDateString("en-US", { weekday: "long", month: "long", day: "numeric" })}
          {:else}Selection{/if}
        </h3>
      </div>
      <button type="button" class="close-btn" aria-label="Close calendar details" onclick={() => calendarStore.closeInspector()}>×</button>
    </div>

    <div class="inspector-content">
      {#if calendarStore.selectedItem}
        {@const item = calendarStore.selectedItem}
        <div class="source-line"><span class={`source-mark ${item.source}`}></span>{sourceLabel(item)}</div>
        <div class="detail-card">
          <span class="detail-label">When</span>
          <strong>{new Date(`${item.startDate}T12:00:00`).toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" })}</strong>
          <span>{formatTime(item)}</span>
        </div>
        {#if item.description}<p class="description">{item.description}</p>{/if}

        {#if item.kind === "task"}
          <div class="detail-card task-progress">
            <span class="detail-label">Tracked</span>
            <TimeDisplay seconds={item.task.total_time_seconds} format="short" />
            {#if item.task.planned_duration_minutes}<span>{item.task.planned_duration_minutes} min planned</span>{/if}
          </div>
          <div class="actions">
            <button type="button" class="secondary" onclick={() => uiStore.openTaskModal({ taskId: item.task.id })}>Open task</button>
            <button type="button" class="primary" onclick={() => calendarStore.toggleTask(item.task.id)}>{item.task.completed ? "Mark active" : "Complete"}</button>
          </div>
        {:else if item.kind === "local_event"}
          <div class="actions">
            <button type="button" class="secondary" onclick={() => calendarStore.editEvent(item.event)}>Edit{item.event.recurrence_rule ? " series" : ""}</button>
            <button type="button" class="danger" onclick={() => deleteSelected(item)}>Delete</button>
          </div>
        {:else if item.kind === "google_event"}
          <p class="read-only-note">Read-only context from your primary Google Calendar.</p>
          {#if item.event.html_link}<button type="button" class="secondary full" onclick={async () => { const { openUrl } = await import("@tauri-apps/plugin-opener"); await openUrl(item.event.html_link!); }}>Open in Google Calendar</button>{/if}
        {:else if item.kind === "time_entry"}
          <div class="detail-card">
            <span class="detail-label">Duration</span>
            <TimeDisplay seconds={item.entry.duration_seconds} format="hms" />
          </div>
          {#if editingTime}
            <label><span>Minutes</span><input class="input" type="number" min="1" bind:value={timeMinutes} /></label>
            <label><span>Note</span><textarea class="input" rows="3" bind:value={timeNote}></textarea></label>
            <div class="actions"><button type="button" class="secondary" onclick={() => (editingTime = false)}>Cancel</button><button type="button" class="primary" onclick={() => saveTime(item)}>Save</button></div>
          {:else}
            <div class="actions"><button type="button" class="secondary" onclick={() => startEditingTime(item)}>Edit</button><button type="button" class="danger" onclick={() => deleteSelected(item)}>Delete</button></div>
          {/if}
        {/if}
      {:else if selectedDateKey}
        <div class="selected-day-content">
          {#if dateItems.length === 0}
            <div class="empty-state"><span aria-hidden="true">○</span><p>No tasks or events scheduled.</p></div>
          {:else}
            <div class="day-items">
              {#each dateItems as item (item.key)}
                <button type="button" class={`day-item ${item.source}`} onclick={() => calendarStore.selectItem(item)}>
                  <span class="item-mark" style={`--item-color:${item.color}`}></span>
                  <span class="item-copy"><strong>{item.title}</strong><small>{formatTime(item)} · {sourceLabel(item)}</small></span>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <div class="day-composer" aria-label="Add to selected day">
          <div class="composer-copy">
            <strong>Add to this day</strong>
            <span>The date will be filled in for you.</span>
          </div>
          <div class="date-actions">
            <button type="button" class="primary" onclick={() => uiStore.openTaskModal({ deadline: selectedDateKey! })}>+ Task</button>
            <button type="button" class="secondary" onclick={() => calendarStore.openNewEvent(selectedDateKey!, null)}>+ Event</button>
          </div>
        </div>
      {/if}
      {#if error}<p class="error" role="alert">{error}</p>{/if}
    </div>
  {/if}
</aside>

<style>
  .inspector { height:100%; display:flex; flex-direction:column; background:var(--bg-secondary); color:var(--text-primary); }
  .inspector-header { display:flex; justify-content:space-between; align-items:flex-start; gap:var(--spacing-md); padding:var(--spacing-lg); border-bottom:1px solid var(--border-light); }
  h3 { margin:3px 0 0; font-size:17px; line-height:1.25; }
  .eyebrow, .detail-label, label > span { color:var(--text-tertiary); font-size:10px; font-weight:700; letter-spacing:.06em; text-transform:uppercase; }
  .close-btn { width:30px; height:30px; border:0; border-radius:50%; background:transparent; color:var(--text-secondary); font-size:22px; cursor:pointer; }
  .close-btn:hover { background:var(--bg-hover); color:var(--text-primary); }
  .inspector-content { flex:1; overflow:auto; padding:var(--spacing-lg); display:flex; flex-direction:column; gap:var(--spacing-md); }
  .source-line { display:flex; align-items:center; gap:7px; color:var(--text-secondary); font-size:12px; }
  .source-mark { width:9px; height:9px; border-radius:50%; background:var(--text-tertiary); }
  .source-mark.tasks { border-radius:2px; background:var(--accent); }
  .source-mark.local { background:var(--success); }
  .source-mark.google { background:#4285f4; box-shadow:inset 0 0 0 2px var(--bg-secondary); outline:1px solid #4285f4; }
  .source-mark.time { border-radius:0; transform:rotate(45deg); }
  .detail-card { display:flex; flex-direction:column; gap:5px; padding:var(--spacing-md); border-radius:var(--radius-md); background:var(--bg-primary); border:1px solid var(--border-light); }
  .detail-card strong { font-size:14px; }
  .detail-card span:not(.detail-label) { color:var(--text-secondary); font-size:12px; }
  .description, .read-only-note { margin:0; color:var(--text-secondary); font-size:13px; line-height:1.5; }
  .actions, .date-actions { display:flex; gap:var(--spacing-sm); }
  .actions button, .date-actions button, .full { flex:1; border-radius:var(--radius-md); padding:8px 10px; font-size:12px; font-weight:600; cursor:pointer; }
  .primary { background:var(--accent); color:var(--accent-contrast); border:1px solid var(--accent); }
  .secondary { background:var(--bg-primary); color:var(--text-primary); border:1px solid var(--border); }
  .danger { background:var(--danger-light); color:var(--danger); border:1px solid transparent; }
  .full { width:100%; }
  label { display:flex; flex-direction:column; gap:5px; }
  .day-items { display:flex; flex-direction:column; gap:5px; }
  .selected-day-content { flex:1; display:flex; flex-direction:column; min-height:120px; }
  .day-composer { position:sticky; bottom:calc(-1 * var(--spacing-lg)); display:flex; flex-direction:column; gap:10px; margin:var(--spacing-sm) calc(-1 * var(--spacing-lg)) calc(-1 * var(--spacing-lg)); padding:var(--spacing-md) var(--spacing-lg); border-top:1px solid var(--border); background:color-mix(in srgb,var(--bg-secondary) 94%,transparent); box-shadow:0 -8px 20px rgba(0,0,0,.06); backdrop-filter:blur(10px); }
  .composer-copy { display:flex; flex-direction:column; gap:2px; }
  .composer-copy strong { color:var(--text-primary); font-size:12px; }
  .composer-copy span { color:var(--text-tertiary); font-size:10px; }
  .day-item { width:100%; display:flex; align-items:center; gap:10px; padding:9px; border:1px solid transparent; border-radius:var(--radius-md); background:var(--bg-primary); color:var(--text-primary); text-align:left; cursor:pointer; }
  .day-item:hover { border-color:var(--border); background:var(--bg-hover); }
  .item-mark { width:4px; height:30px; flex:0 0 auto; border-radius:3px; background:var(--item-color); }
  .google .item-mark { width:7px; border-radius:1px; background:repeating-linear-gradient(135deg,var(--item-color) 0 2px,transparent 2px 4px); }
  .time .item-mark { width:7px; opacity:.7; }
  .item-copy { min-width:0; display:flex; flex-direction:column; gap:3px; }
  .item-copy strong, .item-copy small { white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
  .item-copy strong { font-size:13px; }
  .item-copy small { color:var(--text-tertiary); font-size:10px; }
  .empty-state { margin:auto; text-align:center; color:var(--text-secondary); }
  .empty-state span { font-size:26px; }
  .empty-state p { font-size:13px; }
  .error { color:var(--danger); font-size:12px; }
  :global(body.compact-mode) .inspector-content, :global(body.compact-mode) .inspector-header { padding:var(--spacing-md); }
  :global(body.compact-mode) .day-composer { bottom:calc(-1 * var(--spacing-md)); margin-left:calc(-1 * var(--spacing-md)); margin-right:calc(-1 * var(--spacing-md)); margin-bottom:calc(-1 * var(--spacing-md)); padding:var(--spacing-sm) var(--spacing-md); }
</style>
