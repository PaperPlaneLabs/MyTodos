<script lang="ts">
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import type { TodayTask } from "$lib/services/db";
  import { timerStore } from "$lib/stores/timer.svelte";

  import { formatTodayDeadline } from "./today-view-utils";

  let {
    task,
    overdue = false,
    onEdit,
    onComplete,
    onToggleTimer,
    onContextMenu,
  }: {
    task: TodayTask;
    overdue?: boolean;
    onEdit: (taskId: number) => void;
    onComplete: (taskId: number) => void | Promise<void>;
    onToggleTimer: (taskId: number) => void | Promise<void>;
    onContextMenu: (event: MouseEvent, taskId: number) => void;
  } = $props();

  const deadlineTime = $derived(formatTodayDeadline(task.deadline));
  const isActive = $derived(timerStore.active?.task_id === task.id);
</script>

<div
  class="task-row"
  class:overdue
  class:active-timer={isActive}
  role="group"
  aria-label={task.title}
  oncontextmenu={(event) => onContextMenu(event, task.id)}
>
  <button
    type="button"
    class="complete-btn"
    aria-label={`Mark ${task.title} complete`}
    title="Complete task"
    onclick={() => onComplete(task.id)}
  ><span aria-hidden="true">✓</span></button>
  <button type="button" class="task-main" onclick={() => onEdit(task.id)}>
    <span class="project-dot" style:background={task.project_color ?? "var(--text-tertiary)"}></span>
    <span class="task-copy">
      <strong>{task.title}</strong>
      <small>{task.project_name ?? "No project"}</small>
    </span>
    {#if overdue && task.total_time_seconds > 0}
      <span class="tracked-time"><TimeDisplay seconds={task.total_time_seconds} format="short" /></span>
    {:else if deadlineTime}
      <time>{deadlineTime}</time>
    {/if}
  </button>
  {#if task.project_id}
    <button
      type="button"
      class="timer-btn"
      class:active={isActive}
      aria-label={`${isActive ? (timerStore.isRunning ? "Pause" : "Resume") : "Start"} timer for ${task.title}`}
      title={isActive ? (timerStore.isRunning ? "Pause timer" : "Resume timer") : "Start timer"}
      onclick={() => onToggleTimer(task.id)}
    >{isActive ? (timerStore.isRunning ? "Ⅱ" : "▶") : "◷"}</button>
  {/if}
</div>

<style>
  .task-row { width: 100%; display: flex; align-items: center; gap: var(--spacing-xs); padding: var(--spacing-xs); border-radius: var(--radius-md); background: transparent; color: var(--text-primary); transition: background var(--transition-fast); }
  .task-row:hover, .task-row:focus-within { background: var(--bg-hover); }
  .task-row.active-timer { background: color-mix(in srgb, var(--success-light) 68%, transparent); }
  button { border: 0; cursor: pointer; }
  .task-main { min-width: 0; flex: 1; display: flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-xs); background: transparent; color: inherit; text-align: left; }
  .complete-btn, .timer-btn { display: grid; width: 28px; height: 28px; flex: none; place-items: center; border-radius: var(--radius-sm); color: var(--text-tertiary); background: transparent; }
  .timer-btn { opacity: .62; transition: opacity var(--transition-fast), color var(--transition-fast), background var(--transition-fast); }
  .task-row:hover .timer-btn, .task-row:focus-within .timer-btn, .timer-btn.active { opacity: 1; }
  .complete-btn { border: 1px solid var(--border); border-radius: 50%; }
  .complete-btn span { opacity: 0; }
  .complete-btn:hover { color: var(--success); border-color: var(--success); background: var(--success-light); }
  .complete-btn:hover span, .complete-btn:focus-visible span { opacity: 1; }
  .timer-btn:hover, .timer-btn.active { color: var(--accent); background: var(--accent-light); }
  .project-dot { width: 4px; height: 30px; border-radius: 2px; flex: none; }
  .task-copy { min-width: 0; flex: 1; display: flex; flex-direction: column; }
  .task-copy strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  small, .tracked-time, time { color: var(--text-tertiary); font-size: var(--text-xs); }
  time { color: var(--accent); font-weight: 700; }
  button:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
  :global(body.compact-mode) .task-row { padding: 2px; }
</style>
