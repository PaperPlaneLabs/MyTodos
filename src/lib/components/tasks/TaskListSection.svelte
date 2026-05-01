<script lang="ts">
  import { flip } from "svelte/animate";
  import { tick } from "svelte";
  import { slide } from "svelte/transition";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { windowTrackingStore } from "$lib/stores/window-tracking.svelte";

  let {
    isDragging,
    draggedId,
    formatDeadline,
    isOverdue,
    getDaysRemaining,
    onTaskContextMenu,
    onTaskPointerDown,
    onToggleTimer,
    onStopTimer,
    onOpenResetModal,
  }: {
    isDragging: boolean;
    draggedId: number | null;
    formatDeadline: (deadline: string | null | undefined) => string;
    isOverdue: (deadline: string | null | undefined, completed: boolean) => boolean;
    getDaysRemaining: (updatedAt: number) => string;
    onTaskContextMenu: (event: MouseEvent | PointerEvent, id: number) => void;
    onTaskPointerDown: (event: PointerEvent, id: number, index: number) => void;
    onToggleTimer: (taskId: number) => void | Promise<void>;
    onStopTimer: () => void | Promise<void>;
    onOpenResetModal: (taskId: number) => void;
  } = $props();

  let windowTrackingNoticeVisible = $state(false);
  let windowTrackingNoticeTimeout: number | null = null;

  function showWindowTrackingNotice(): void {
    windowTrackingNoticeVisible = true;
    if (windowTrackingNoticeTimeout !== null) {
      clearTimeout(windowTrackingNoticeTimeout);
    }

    windowTrackingNoticeTimeout = window.setTimeout(() => {
      windowTrackingNoticeVisible = false;
      windowTrackingNoticeTimeout = null;
    }, 3200);
  }

  function handleTaskBodyPointerDown(event: PointerEvent, taskId: number, index: number): void {
    const target = event.target as HTMLElement;
    if (target.closest(".checkbox-container") || target.closest(".task-controls")) {
      return;
    }
    onTaskPointerDown(event, taskId, index);
  }

  function handleTaskDragHandlePointerDown(
    event: PointerEvent,
    taskId: number,
    index: number,
  ): void {
    event.stopPropagation();
    onTaskPointerDown(event, taskId, index);
  }

  async function focusTaskReorderHandle(taskId: number) {
    await tick();
    document
      .querySelector<HTMLElement>(`[data-task-handle-id="${taskId}"]`)
      ?.focus();
  }

  async function handleTaskReorderKeydown(
    event: KeyboardEvent,
    taskId: number,
    index: number,
  ) {
    const offset =
      event.key === "ArrowUp" ? -1 : event.key === "ArrowDown" ? 1 : 0;
    if (offset === 0) return;

    const nextIndex = Math.max(
      0,
      Math.min(taskStore.tasks.length - 1, index + offset),
    );
    if (nextIndex === index) return;

    event.preventDefault();
    taskStore.reorderLocal(index, nextIndex);
    await taskStore.reorder(taskStore.tasks.map((item) => item.id));
    await focusTaskReorderHandle(taskId);
  }

  function getTaskCheckboxLabel(title: string, completed: boolean) {
    return completed
      ? `Mark ${title} as not completed`
      : `Mark ${title} as completed`;
  }

  function getTimerActionLabel(
    action: "pause" | "resume" | "stop" | "start" | "reset-session" | "reset-total",
    title: string,
  ) {
    switch (action) {
      case "pause":
        return `Pause timer for ${title}`;
      case "resume":
        return `Resume timer for ${title}`;
      case "stop":
        return `Stop timer for ${title}`;
      case "start":
        return `Start timer for ${title}`;
      case "reset-session":
        return `Discard the paused timer session for ${title}`;
      case "reset-total":
        return `Reset all tracked time for ${title}`;
    }
  }

  function handleStartTimerClick(taskId: number): void {
    if (windowTrackingStore.enabled) {
      showWindowTrackingNotice();
      return;
    }

    void onToggleTimer(taskId);
  }
</script>

<div class="tasks-section">
  <div class="section-header">
    <h2>{projectStore.selected?.name || "Tasks"}</h2>
    <button
      type="button"
      class="btn btn-ghost btn-sm"
      onclick={() => uiStore.openTaskModal()}
    >
      + Task
    </button>
  </div>

  <div class="tasks-list" role="list">
    {#if windowTrackingNoticeVisible}
      <div class="window-tracking-notice" role="status">
        Window tracking is on, so project/task timers are disabled.
      </div>
    {/if}

    {#each taskStore.activeTasks as task, index (task.id)}
      <div
        animate:flip={{ duration: 300 }}
        transition:slide={{ duration: 200 }}
        class="task-item-wrapper"
        data-index={index}
        class:dragging={isDragging && draggedId === task.id}
      >
        <div
          class="task-item"
          role="group"
          aria-labelledby={"task-title-" + task.id}
          class:task-timer-active={timerStore.active?.task_id === task.id &&
            timerStore.isRunning}
          class:task-timer-paused={timerStore.active?.task_id === task.id &&
            !timerStore.isRunning}
          oncontextmenu={(event) => onTaskContextMenu(event, task.id)}
          onpointerdown={(event) => handleTaskBodyPointerDown(event, task.id, index)}
        >
          <div
            class="drag-handle-task"
            role="button"
            tabindex="0"
            data-task-handle-id={task.id}
            aria-label={"Reorder task " + task.title + ". Use Up or Down arrow keys."}
            onkeydown={(event) => handleTaskReorderKeydown(event, task.id, index)}
            onpointerdown={(event) => handleTaskDragHandlePointerDown(event, task.id, index)}
          >
            ⋮⋮
          </div>
          <label class="checkbox-container">
            <input
              type="checkbox"
              checked={task.completed}
              aria-label={getTaskCheckboxLabel(task.title, task.completed)}
              onchange={() => taskStore.toggleCompletion(task.id)}
              class="task-checkbox-hidden"
            />
            <span class="checkbox-custom"></span>
          </label>
          <div class="task-content">
            <div
              class="task-title"
              id={"task-title-" + task.id}
              class:completed={task.completed}
            >
              {task.title}
            </div>
            <div class="task-meta">
              {#if task.deadline}
                <div
                  class="task-deadline text-xs"
                  class:overdue={isOverdue(task.deadline, task.completed)}
                >
                  <span class="deadline-icon">📅</span>
                  {formatDeadline(task.deadline)}
                </div>
              {/if}
              {#if task.total_time_seconds > 0 && task.project_id}
                <div class="task-time text-xs">
                  <span class="time-icon">⏱</span>
                  <TimeDisplay seconds={task.total_time_seconds} />
                </div>
              {/if}
              {#if timerStore.active && timerStore.active.task_id === task.id}
                <div
                  class="inline-timer"
                  class:running={timerStore.isRunning}
                  class:paused={!timerStore.isRunning}
                >
                  <span class="timer-indicator"></span>
                  <TimeDisplay seconds={Math.floor(timerStore.elapsed)} format="short" />
                </div>
              {/if}
            </div>
          </div>
          <div class="task-controls">
            {#if timerStore.active && timerStore.active.task_id === task.id}
              <button
                type="button"
                class="btn-icon-compact"
                onclick={() => (timerStore.isRunning ? timerStore.pause() : timerStore.resume())}
                aria-label={getTimerActionLabel(
                  timerStore.isRunning ? "pause" : "resume",
                  task.title,
                )}
                title={timerStore.isRunning ? "Pause timer" : "Resume timer"}
              >
                {#if timerStore.isRunning}⏸{:else}▶{/if}
              </button>
              <button
                type="button"
                class="btn-icon-compact btn-stop"
                onclick={onStopTimer}
                aria-label={getTimerActionLabel("stop", task.title)}
                title="Stop timer and save"
              >
                ⏹
              </button>
              {#if !timerStore.isRunning}
                <button
                  type="button"
                  class="btn-icon-compact btn-reset"
                  onclick={() => onOpenResetModal(task.id)}
                  aria-label={getTimerActionLabel("reset-session", task.title)}
                  title="Reset timer (discard time)"
                >
                  ⟲
                </button>
              {/if}
            {:else if task.project_id}
              <button
                type="button"
                class="btn-icon-compact"
                class:window-tracking-disabled={windowTrackingStore.enabled}
                onclick={() => handleStartTimerClick(task.id)}
                aria-label={getTimerActionLabel("start", task.title)}
                title={windowTrackingStore.enabled
                  ? "Window tracking is on, so project/task timers are disabled."
                  : "Start timer"}
              >
                ⏱
              </button>
              {#if task.total_time_seconds > 0}
                <button
                  type="button"
                  class="btn-icon-compact btn-reset"
                  onclick={() => onOpenResetModal(task.id)}
                  aria-label={getTimerActionLabel("reset-total", task.title)}
                  title="Reset all time for this task"
                >
                  ⟲
                </button>
              {/if}
            {/if}
          </div>
        </div>
      </div>
    {/each}

    {#if taskStore.completedTasks.length > 0}
      <button
        type="button"
        class="completed-separator"
        transition:slide={{ duration: 200 }}
        onclick={() => uiStore.toggleCompletedTasks()}
        aria-controls="completed-tasks-panel"
        aria-expanded={!uiStore.completedTasksCollapsed}
      >
        <span class="completed-caret" class:expanded={!uiStore.completedTasksCollapsed}>
          <svg
            width="10"
            height="10"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="m9 18 6-6-6-6" />
          </svg>
        </span>
        <span class="completed-label">Completed</span>
        <div class="completed-line"></div>
        <span class="completed-count">{taskStore.completedTasks.length}</span>
      </button>

      {#if !uiStore.completedTasksCollapsed}
        <div
          id="completed-tasks-panel"
          class="completed-tasks-container"
          transition:slide={{ duration: 250 }}
        >
          {#each taskStore.completedTasks as task, index (task.id)}
            {@const globalIndex = index + taskStore.activeTasks.length}
            <div
              animate:flip={{ duration: 300 }}
              class="task-item-wrapper"
              data-index={globalIndex}
              class:dragging={isDragging && draggedId === task.id}
            >
              <div
                class="task-item completed-task-item"
                role="group"
                aria-labelledby={"task-title-" + task.id}
                oncontextmenu={(event) => onTaskContextMenu(event, task.id)}
                onpointerdown={(event) => handleTaskBodyPointerDown(event, task.id, globalIndex)}
              >
                <div
                  class="drag-handle-task"
                  role="button"
                  tabindex="0"
                  data-task-handle-id={task.id}
                  aria-label={"Reorder task " + task.title + ". Use Up or Down arrow keys."}
                  onkeydown={(event) =>
                    handleTaskReorderKeydown(event, task.id, globalIndex)}
                  onpointerdown={(event) =>
                    handleTaskDragHandlePointerDown(event, task.id, globalIndex)}
                >
                  ⋮⋮
                </div>
                <label class="checkbox-container">
                  <input
                    type="checkbox"
                    checked={task.completed}
                    aria-label={getTaskCheckboxLabel(task.title, task.completed)}
                    onchange={() => taskStore.toggleCompletion(task.id)}
                    class="task-checkbox-hidden"
                  />
                  <span class="checkbox-custom"></span>
                </label>
                <div class="task-content">
                  <div
                    class="task-title"
                    id={"task-title-" + task.id}
                    class:completed={task.completed}
                  >
                    {task.title}
                  </div>
                  <div class="task-meta">
                    {#if task.deadline}
                      <div
                        class="task-deadline text-xs"
                        class:overdue={isOverdue(task.deadline, task.completed)}
                      >
                        <span class="deadline-icon">📅</span>
                        {formatDeadline(task.deadline)}
                      </div>
                    {/if}
                    {#if task.total_time_seconds > 0 && task.project_id}
                      <div class="task-time text-xs">
                        <span class="time-icon">⏱</span>
                        <TimeDisplay seconds={task.total_time_seconds} />
                      </div>
                    {/if}
                    <div
                      class="task-days-remaining text-xs text-tertiary"
                      style="margin-left: auto;"
                      title="Completed tasks are auto-deleted after 30 days"
                    >
                      {getDaysRemaining(task.updated_at)}
                    </div>
                  </div>
                </div>
                <div class="task-controls">
                  {#if task.project_id && task.total_time_seconds > 0}
                    <button
                      type="button"
                      class="btn-icon-compact btn-reset"
                      onclick={() => onOpenResetModal(task.id)}
                      aria-label={getTimerActionLabel("reset-total", task.title)}
                      title="Reset all time for this task"
                    >
                      ⟲
                    </button>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    {/if}

    {#if taskStore.tasks.length === 0}
      <div class="empty-state">
        <p class="text-secondary text-sm">No tasks yet</p>
        <p class="text-tertiary text-xs">Click "+ Task" to create your first task</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--spacing-md);
    padding-bottom: var(--spacing-sm);
    border-bottom: 2px solid var(--border-light);
  }

  .section-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .tasks-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .window-tracking-notice {
    padding: 8px 10px;
    border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--accent) 10%, var(--bg-secondary));
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
  }

  .completed-separator {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) 0 var(--spacing-sm);
    user-select: none;
    background: none;
    border: none;
    width: 100%;
    cursor: pointer;
    transition: opacity var(--transition-fast);
  }

  .completed-separator:hover {
    opacity: 0.8;
  }

  .completed-caret {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    transition: transform var(--transition-normal);
  }

  .completed-caret.expanded {
    transform: rotate(90deg);
  }

  .completed-label {
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
    white-space: nowrap;
  }

  .completed-line {
    flex: 1;
    height: 1px;
    background: linear-gradient(90deg, var(--border) 0%, transparent 100%);
  }

  .completed-count {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-tertiary);
    background-color: var(--bg-secondary);
    padding: 2px 6px;
    border-radius: 10px;
    border: 1px solid var(--border);
  }

  .task-item-wrapper {
    transition: transform 0.2s ease;
    border-radius: var(--radius-md);
    min-height: 10px;
    user-select: none;
    touch-action: none;
  }

  .dragging {
    z-index: 100;
    opacity: 0.9;
    transform: scale(1.02);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
    pointer-events: none;
  }

  .drag-handle-task {
    cursor: grab;
    color: var(--text-tertiary);
    opacity: 0;
    font-size: 14px;
    padding: 0 2px;
    transition: opacity 0.2s;
    flex-shrink: 0;
    margin-right: 4px;
    margin-left: -4px;
  }

  .task-item:hover .drag-handle-task {
    opacity: 1;
  }

  .drag-handle-task:focus-visible {
    opacity: 1;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all var(--transition-normal);
    cursor: grab;
    user-select: none;
  }

  .task-item:active {
    cursor: grabbing;
  }

  .task-item:hover {
    background-color: var(--bg-hover);
    border-color: var(--text-tertiary);
  }

  .task-item.task-timer-active {
    border-color: var(--timer-active);
    background-color: var(--success-light);
    box-shadow: 0 0 0 3px var(--success-glow);
  }

  .task-item.task-timer-paused {
    border-color: var(--timer-paused);
    background-color: var(--warning-light);
    box-shadow: 0 0 0 3px var(--warning-glow);
  }

  .checkbox-container {
    display: block;
    position: relative;
    width: 20px;
    height: 20px;
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
  }

  .task-checkbox-hidden {
    position: absolute;
    opacity: 0;
    cursor: pointer;
    height: 0;
    width: 0;
  }

  .checkbox-custom {
    position: absolute;
    top: 0;
    left: 0;
    height: 20px;
    width: 20px;
    background-color: var(--bg-primary);
    border: 2px solid var(--border);
    border-radius: 6px;
    transition: all var(--transition-fast);
  }

  .checkbox-container:hover input ~ .checkbox-custom {
    border-color: var(--accent);
    background-color: var(--bg-hover);
  }

  .checkbox-container input:checked ~ .checkbox-custom {
    background-color: var(--accent);
    border-color: var(--accent);
  }

  .checkbox-container input:focus-visible ~ .checkbox-custom {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .checkbox-custom:after {
    content: "";
    position: absolute;
    display: none;
  }

  .checkbox-container input:checked ~ .checkbox-custom:after {
    display: block;
  }

  .checkbox-container .checkbox-custom:after {
    left: 6px;
    top: 2px;
    width: 5px;
    height: 10px;
    border: solid var(--accent-contrast);
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }

  .task-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .task-title {
    font-weight: 500;
    transition: all var(--transition-fast);
    line-height: 1.3;
  }

  .task-title.completed {
    text-decoration: line-through;
    color: var(--text-secondary);
    opacity: 0.6;
  }

  .task-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-wrap: wrap;
  }

  .task-time {
    display: flex;
    align-items: center;
    gap: 3px;
    color: var(--text-secondary);
  }

  .task-deadline {
    display: flex;
    align-items: center;
    gap: 3px;
    color: var(--text-secondary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
  }

  .task-deadline.overdue {
    background: var(--danger-light);
    color: var(--danger);
  }

  .time-icon {
    opacity: 0.5;
    font-size: 10px;
  }

  .deadline-icon {
    opacity: 0.5;
    font-size: 10px;
  }

  .inline-timer {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
    font-family: var(--font-mono);
    transition: all var(--transition-fast);
  }

  .inline-timer.running {
    background-color: var(--success);
    color: var(--success-contrast);
    animation: pulse 2s ease-in-out infinite;
  }

  .inline-timer.paused {
    background-color: var(--warning);
    color: var(--warning-contrast);
  }

  .timer-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: currentColor;
  }

  .inline-timer.running .timer-indicator {
    animation: blink 1s ease-in-out infinite;
  }

  .completed-tasks-container {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .completed-task-item {
    opacity: 0.7;
    filter: grayscale(0.5);
    transition: all var(--transition-normal);
  }

  .completed-task-item:hover {
    opacity: 1;
    filter: grayscale(0);
  }

  .task-controls {
    display: flex;
    gap: 4px;
    align-items: center;
    flex-shrink: 0;
  }

  .btn-icon-compact {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
    font-size: 13px;
    background-color: transparent;
    border: 1px solid transparent;
  }

  .btn-icon-compact:hover {
    background-color: var(--accent-light);
    color: var(--accent);
    border-color: var(--accent);
  }

  .btn-icon-compact.window-tracking-disabled {
    opacity: 0.65;
    border-color: var(--border);
  }

  .btn-icon-compact.btn-stop:hover {
    background-color: var(--danger-light);
    color: var(--danger);
    border-color: var(--danger);
  }

  .btn-icon-compact.btn-reset {
    color: var(--warning);
    border-color: var(--border);
  }

  .btn-icon-compact.btn-reset:hover {
    background-color: var(--warning-light);
    color: var(--warning);
    border-color: var(--warning);
    transform: scale(1.1);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-xl) var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    align-items: center;
    background-color: var(--bg-secondary);
    border: 2px dashed var(--border);
    border-radius: var(--radius-md);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }

    50% {
      opacity: 0.85;
    }
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 1;
    }

    50% {
      opacity: 0.3;
    }
  }
</style>
