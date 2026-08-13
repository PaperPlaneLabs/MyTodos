<script lang="ts">
  import { untrack } from "svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import type { EventEditorDraft, RecurrenceFrequency } from "$lib/types/calendar";

  let { draft } = $props<{ draft: EventEditorDraft }>();
  let form = $state<EventEditorDraft>(untrack(() => ({
    ...draft,
    recurrence: draft.recurrence ? { ...draft.recurrence, weekdays: [...draft.recurrence.weekdays] } : null,
  })));
  let saving = $state(false);
  let error = $state<string | null>(null);
  let recurrenceEnd = $state<"never" | "date" | "count">(
    form.recurrence?.until ? "date" : form.recurrence?.count ? "count" : "never",
  );

  const colors = ["#6366f1", "#0ea5e9", "#10b981", "#f59e0b", "#ef4444", "#a855f7"];
  const weekdayOptions = [
    { value: 1, label: "M" }, { value: 2, label: "T" },
    { value: 3, label: "W" }, { value: 4, label: "T" },
    { value: 5, label: "F" }, { value: 6, label: "S" },
    { value: 0, label: "S" },
  ];

  function setFrequency(value: string) {
    if (value === "none") {
      form.recurrence = null;
      return;
    }
    form.recurrence = {
      frequency: value as RecurrenceFrequency,
      interval: form.recurrence?.interval ?? 1,
      weekdays: form.recurrence?.weekdays.length
        ? [...form.recurrence.weekdays]
        : [new Date(`${form.startDate}T12:00:00`).getDay()],
      until: null,
      count: null,
    };
    recurrenceEnd = "never";
  }

  function toggleWeekday(day: number) {
    if (!form.recurrence) return;
    const weekdays = form.recurrence.weekdays.includes(day)
      ? form.recurrence.weekdays.filter((value) => value !== day)
      : [...form.recurrence.weekdays, day];
    form.recurrence = { ...form.recurrence, weekdays };
  }

  function setRecurrenceEnd(value: "never" | "date" | "count") {
    recurrenceEnd = value;
    if (!form.recurrence) return;
    form.recurrence = {
      ...form.recurrence,
      until: value === "date" ? form.recurrence.until ?? form.endDate : null,
      count: value === "count" ? form.recurrence.count ?? 10 : null,
    };
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!form.title.trim()) {
      error = "Give the event a title.";
      return;
    }
    if (form.endDate < form.startDate) {
      error = "The end date cannot be before the start date.";
      return;
    }
    if (form.recurrence?.frequency === "weekly" && form.recurrence.weekdays.length === 0) {
      error = "Choose at least one weekday.";
      return;
    }
    saving = true;
    error = null;
    try {
      await calendarStore.saveEvent(form);
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    } finally {
      saving = false;
    }
  }
</script>

<form class="event-editor" onsubmit={submit}>
  <div class="editor-heading">
    <div>
      <span class="eyebrow">{form.id === null ? "New event" : "Edit series"}</span>
      <h3>{form.id === null ? "Add to calendar" : "Event details"}</h3>
    </div>
    <button type="button" class="icon-btn" aria-label="Close event editor" onclick={() => calendarStore.closeEventEditor()}>×</button>
  </div>

  <label>
    <span>Title</span>
    <input class="input" bind:value={form.title} placeholder="What is happening?" />
  </label>

  <label>
    <span>Description</span>
    <textarea class="input" rows="3" bind:value={form.description} placeholder="Optional notes"></textarea>
  </label>

  <label class="all-day-row">
    <span>All day</span>
    <input type="checkbox" bind:checked={form.isAllDay} />
  </label>

  <div class="field-grid">
    <label><span>Starts</span><input class="input" type="date" bind:value={form.startDate} /></label>
    {#if !form.isAllDay}
      <label><span>Time</span><input class="input" type="time" bind:value={form.startTime} /></label>
    {/if}
    <label><span>Ends</span><input class="input" type="date" min={form.startDate} bind:value={form.endDate} /></label>
    {#if !form.isAllDay}
      <label><span>Time</span><input class="input" type="time" bind:value={form.endTime} /></label>
    {/if}
  </div>

  <fieldset>
    <legend>Repeat</legend>
    <div class="repeat-row">
      <select class="input" value={form.recurrence?.frequency ?? "none"} onchange={(event) => setFrequency(event.currentTarget.value)}>
        <option value="none">Does not repeat</option>
        <option value="daily">Daily</option>
        <option value="weekly">Weekly</option>
        <option value="monthly">Monthly</option>
        <option value="yearly">Yearly</option>
      </select>
      {#if form.recurrence}
        <label class="interval"><span>Every</span><input class="input" type="number" min="1" max="999" bind:value={form.recurrence.interval} /></label>
      {/if}
    </div>
    {#if form.recurrence?.frequency === "weekly"}
      <div class="weekday-picker" aria-label="Repeat on weekdays">
        {#each weekdayOptions as day}
          <button type="button" class:active={form.recurrence.weekdays.includes(day.value)} aria-pressed={form.recurrence.weekdays.includes(day.value)} onclick={() => toggleWeekday(day.value)}>{day.label}</button>
        {/each}
      </div>
    {/if}
    {#if form.recurrence}
      <div class="repeat-end">
        <select class="input" value={recurrenceEnd} onchange={(event) => setRecurrenceEnd(event.currentTarget.value as typeof recurrenceEnd)}>
          <option value="never">Never ends</option>
          <option value="date">Ends on date</option>
          <option value="count">Ends after count</option>
        </select>
        {#if recurrenceEnd === "date"}
          <input class="input" type="date" min={form.startDate} bind:value={form.recurrence.until} />
        {:else if recurrenceEnd === "count"}
          <input class="input" type="number" min="1" max="999" bind:value={form.recurrence.count} aria-label="Occurrence count" />
        {/if}
      </div>
    {/if}
  </fieldset>

  <fieldset>
    <legend>Color</legend>
    <div class="color-picker">
      {#each colors as color}
        <button type="button" class:active={form.color === color} aria-label={`Use ${color}`} aria-pressed={form.color === color} style={`--event-color:${color}`} onclick={() => (form.color = color)}></button>
      {/each}
    </div>
  </fieldset>

  {#if error}<p class="form-error" role="alert">{error}</p>{/if}
  <div class="editor-actions">
    <button type="button" class="btn-secondary" onclick={() => calendarStore.closeEventEditor()}>Cancel</button>
    <button type="submit" class="btn-primary" disabled={saving}>{saving ? "Saving…" : form.id === null ? "Create event" : "Save series"}</button>
  </div>
</form>

<style>
  .event-editor { display:flex; flex-direction:column; gap:var(--spacing-md); padding:var(--spacing-lg); overflow:auto; height:100%; background:var(--bg-secondary); }
  .editor-heading { display:flex; align-items:flex-start; justify-content:space-between; gap:var(--spacing-md); }
  h3 { margin:2px 0 0; font-size:18px; color:var(--text-primary); }
  .eyebrow, label > span, legend { color:var(--text-tertiary); font-size:11px; font-weight:700; letter-spacing:.05em; text-transform:uppercase; }
  label { display:flex; flex-direction:column; gap:6px; }
  textarea { resize:vertical; min-height:70px; }
  .icon-btn { width:30px; height:30px; border:0; border-radius:50%; background:transparent; color:var(--text-secondary); font-size:22px; cursor:pointer; }
  .icon-btn:hover { background:var(--bg-hover); color:var(--text-primary); }
  .all-day-row { flex-direction:row; align-items:center; justify-content:space-between; padding:var(--spacing-sm) 0; }
  .field-grid { display:grid; grid-template-columns:1fr 110px; gap:var(--spacing-sm); }
  fieldset { border:0; border-top:1px solid var(--border-light); margin:0; padding:var(--spacing-md) 0 0; }
  legend { padding:0 var(--spacing-xs) 0 0; }
  .repeat-row, .repeat-end { display:flex; gap:var(--spacing-sm); align-items:flex-end; }
  .repeat-row > select, .repeat-end > select { flex:1; }
  .interval { width:80px; }
  .weekday-picker { display:flex; gap:5px; margin-top:var(--spacing-sm); }
  .weekday-picker button, .color-picker button { border:1px solid var(--border); cursor:pointer; }
  .weekday-picker button { width:30px; height:30px; border-radius:50%; background:var(--bg-primary); color:var(--text-secondary); }
  .weekday-picker button.active { background:var(--accent); color:var(--accent-contrast); border-color:var(--accent); }
  .repeat-end { margin-top:var(--spacing-sm); }
  .color-picker { display:flex; gap:8px; }
  .color-picker button { width:25px; height:25px; border-radius:50%; background:var(--event-color); }
  .color-picker button.active { outline:2px solid var(--text-primary); outline-offset:2px; }
  .form-error { margin:0; color:var(--danger); font-size:12px; }
  .editor-actions { display:flex; gap:var(--spacing-sm); margin-top:auto; padding-top:var(--spacing-md); }
  .editor-actions button { flex:1; padding:9px 12px; border-radius:var(--radius-md); font-weight:600; cursor:pointer; }
  .btn-secondary { color:var(--text-primary); background:var(--bg-primary); border:1px solid var(--border); }
  .btn-primary { color:var(--accent-contrast); background:var(--accent); border:1px solid var(--accent); }
  @media (max-width:480px) { .field-grid { grid-template-columns:1fr 100px; } }
</style>
