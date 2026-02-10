<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { db } from "$lib/services/db";
  import type { TimeEntryWithTask } from "$lib/types/calendar";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";

  let { entry } = $props<{ entry: TimeEntryWithTask }>();

  async function handleDelete() {
    if (confirm("Are you sure you want to delete this time entry?")) {
      await db.timeEntries.delete(entry.id);
      uiStore.selectCalendarEntry(null);
      await calendarStore.refreshCurrentRange();
    }
  }

  function formatTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' });
  }
</script>

<div class="time-entry-panel">
  <div class="panel-header">
    <h3>Time Entry</h3>
    <button class="close-btn" onclick={() => uiStore.selectCalendarEntry(null)} aria-label="Close">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6 6 18M6 6l12 12" />
      </svg>
    </button>
  </div>

  <div class="panel-content">
    <div class="entry-task">
      <span class="task-icon">⏱</span>
      <div class="task-info">
        <span class="task-title">{entry.task_title}</span>
        {#if entry.project_name}
          <span class="project-tag" style="color: {entry.project_color || 'var(--text-secondary)'}">
            {entry.project_name}
          </span>
        {/if}
      </div>
    </div>

    <div class="entry-duration">
      <TimeDisplay seconds={entry.duration_seconds} format="hms" />
    </div>

    <div class="entry-time-range">
      <div class="time-item">
        <span class="time-label">Started</span>
        <span class="time-value">{formatTime(entry.started_at)}</span>
      </div>
      <div class="time-item">
        <span class="time-label">Ended</span>
        <span class="time-value">{formatTime(entry.ended_at)}</span>
      </div>
      <div class="time-item">
        <span class="time-label">Date</span>
        <span class="time-value">{formatDate(entry.started_at)}</span>
      </div>
    </div>

    {#if entry.note}
      <div class="entry-note">
        <span class="note-label">Note</span>
        <p class="note-content">{entry.note}</p>
      </div>
    {/if}

    <div class="entry-actions">
      <button class="btn btn-secondary btn-sm" onclick={() => {
        uiStore.selectCalendarEntry(null);
        uiStore.openTaskModal({ taskId: entry.task_id });
      }}>
        View Task
      </button>
      <button class="btn btn-danger btn-sm" onclick={handleDelete}>
        Delete
      </button>
    </div>
  </div>
</div>

<style>
  .time-entry-panel {
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border);
    background: var(--bg-tertiary);
  }

  .panel-header h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: var(--spacing-xs);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .panel-content {
    flex: 1;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    overflow-y: auto;
  }

  .entry-task {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .task-icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .task-info {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .task-title {
    font-weight: 600;
    font-size: 15px;
    color: var(--text-primary);
    line-height: 1.3;
  }

  .project-tag {
    font-size: 12px;
    font-weight: 500;
  }

  .entry-duration {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-md);
    background: linear-gradient(135deg, var(--accent-light) 0%, var(--bg-primary) 100%);
    border-radius: var(--radius-md);
    border: 1px solid var(--accent);
  }

  .entry-duration :global(.time-display) {
    font-size: 28px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
  }

  .entry-time-range {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .time-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .time-label {
    font-size: 12px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .time-value {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .entry-note {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
  }

  .note-label {
    font-size: 12px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .note-content {
    font-size: 14px;
    color: var(--text-primary);
    line-height: 1.5;
    margin: 0;
  }

  .entry-actions {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: auto;
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--border);
  }

  .btn {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
    border: none;
  }

  .btn-sm {
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: 12px;
  }

  .btn-secondary {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
    border-color: var(--accent);
  }

  .btn-danger {
    background: var(--danger-light);
    color: var(--danger);
  }

  .btn-danger:hover {
    background: var(--danger);
    color: white;
  }
</style>
