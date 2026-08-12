<script lang="ts">
  import type { CalendarEvent } from "$lib/services/db";
  import { formatEventTime } from "./today-view-utils";
  let { event, onOpen }: { event: CalendarEvent; onOpen: () => void } = $props();
</script>

<button type="button" class="event-row" style:border-left-color={event.color || "var(--accent)"} aria-label={`Open ${event.title} in Calendar`} onclick={onOpen}>
  <span class="calendar-marker" aria-hidden="true">▦</span>
  <time>{formatEventTime(event)}</time>
  <span class="event-copy"><strong>{event.title}</strong>{#if event.description}<small>{event.description}</small>{/if}</span>
</button>

<style>
  .event-row { width: 100%; display: flex; align-items: flex-start; gap: var(--spacing-sm); padding: var(--spacing-sm) var(--spacing-xs); border: 0; border-left: 2px solid var(--accent); border-radius: 0 var(--radius-sm) var(--radius-sm) 0; background: transparent; color: var(--text-primary); text-align: left; cursor: pointer; }
  .event-row:hover, .event-row:focus-visible { background: var(--bg-hover); }
  .event-row:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  .calendar-marker { flex: none; color: var(--text-tertiary); line-height: 1.2; }
  time { width: 54px; flex: none; color: var(--text-secondary); font-size: var(--text-xs); font-weight: 700; }
  .event-copy { min-width: 0; display: flex; flex-direction: column; }
  strong, small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small { color: var(--text-tertiary); font-size: var(--text-xs); }
  :global(body.compact-mode) .event-row { padding: var(--spacing-xs) var(--spacing-sm); }
</style>
