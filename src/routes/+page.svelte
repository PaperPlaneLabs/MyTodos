<script lang="ts">
  import { onMount } from "svelte";
  import "$lib/styles/global.css";
  import AppHeader from "$lib/components/layout/AppHeader.svelte";
  import Modal from "$lib/components/common/Modal.svelte";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import type { Task } from "$lib/services/db";

  let projectName = $state("");
  let taskTitle = $state("");
  let selectedTaskForTimer = $state<number | null>(null);
  let showResetModal = $state(false);

  onMount(async () => {
    uiStore.initTheme();
    await projectStore.loadAll();
    await timerStore.loadActive();

    if (projectStore.projects.length > 0 && !projectStore.selectedId) {
      projectStore.setSelected(projectStore.projects[0].id);
      await taskStore.loadByProject(projectStore.projects[0].id);
    }
  });

  $effect(() => {
    if (projectStore.selectedId) {
      taskStore.loadByProject(projectStore.selectedId);
    }
  });

  async function handleCreateProject() {
    if (!projectName.trim()) return;
    await projectStore.create(projectName);
    projectName = "";
    uiStore.closeProjectModal();
  }

  async function handleCreateTask() {
    if (!taskTitle.trim() || !projectStore.selectedId) return;
    await taskStore.createTask(projectStore.selectedId, null, taskTitle);
    taskTitle = "";
    uiStore.closeTaskModal();
  }

  async function handleToggleTimer(taskId: number) {
    if (timerStore.active && timerStore.active.task_id === taskId) {
      if (timerStore.isRunning) {
        await timerStore.pause();
      } else {
        await timerStore.resume();
      }
    } else {
      if (timerStore.active) {
        await timerStore.stop();
      }
      await timerStore.start(taskId);
    }
  }

  async function handleStopTimer() {
    await timerStore.stop();
    if (projectStore.selectedId) {
      await taskStore.loadByProject(projectStore.selectedId);
    }
  }

  function openResetModal() {
    showResetModal = true;
  }

  async function handleResetTimer() {
    await timerStore.reset();
    showResetModal = false;
  }
</script>

<div class="app-container">
  <AppHeader />

  <div class="main-content">
    <div class="projects-section">
      <div class="section-header">
        <h2>Projects</h2>
        <button class="btn btn-ghost btn-sm" onclick={() => uiStore.openProjectModal()}>
          + New
        </button>
      </div>

      <div class="projects-list">
        {#each projectStore.projects as project (project.id)}
          <button
            class="project-item"
            class:active={projectStore.selectedId === project.id}
            onclick={() => projectStore.setSelected(project.id)}
          >
            <div class="project-color" style="background-color: {project.color}"></div>
            <div class="project-info">
              <div class="project-header">
                <div class="project-name">{project.name}</div>
                {#if project.total_time_seconds > 0}
                  <div class="project-time-badge">
                    <TimeDisplay seconds={project.total_time_seconds} />
                  </div>
                {/if}
              </div>
            </div>
          </button>
        {/each}

        {#if projectStore.projects.length === 0}
          <div class="empty-state">
            <p class="text-secondary text-sm">No projects yet</p>
            <p class="text-tertiary text-xs">Click "+ New" to create your first project</p>
            <button class="btn btn-primary btn-sm" onclick={() => uiStore.openProjectModal()}>
              Create Project
            </button>
          </div>
        {/if}
      </div>
    </div>

    {#if projectStore.selected}
      <div class="tasks-section">
        <div class="section-header">
          <h2>{projectStore.selected.name}</h2>
          <button class="btn btn-ghost btn-sm" onclick={() => uiStore.openTaskModal()}>
            + Task
          </button>
        </div>

        <div class="tasks-list">
          {#each taskStore.tasks as task (task.id)}
            <div
              class="task-item"
              class:task-timer-active={timerStore.active?.task_id === task.id && timerStore.isRunning}
              class:task-timer-paused={timerStore.active?.task_id === task.id && !timerStore.isRunning}
            >
              <input
                type="checkbox"
                checked={task.completed}
                onchange={() => taskStore.toggleCompletion(task.id)}
                class="task-checkbox"
              />
              <div class="task-content">
                <div class="task-title" class:completed={task.completed}>{task.title}</div>
                <div class="task-meta">
                  {#if task.total_time_seconds > 0}
                    <div class="task-time text-xs">
                      <span class="time-icon">⏱</span>
                      <TimeDisplay seconds={task.total_time_seconds} />
                    </div>
                  {/if}
                  {#if timerStore.active && timerStore.active.task_id === task.id}
                    <div class="inline-timer" class:running={timerStore.isRunning} class:paused={!timerStore.isRunning}>
                      <span class="timer-indicator"></span>
                      <TimeDisplay seconds={Math.floor(timerStore.elapsed)} format="short" />
                    </div>
                  {/if}
                </div>
              </div>
              <div class="task-controls">
                {#if timerStore.active && timerStore.active.task_id === task.id}
                  <button
                    class="btn-icon-compact"
                    onclick={() => timerStore.isRunning ? timerStore.pause() : timerStore.resume()}
                    title={timerStore.isRunning ? "Pause timer" : "Resume timer"}
                  >
                    {#if timerStore.isRunning}
                      ⏸
                    {:else}
                      ▶
                    {/if}
                  </button>
                  <button
                    class="btn-icon-compact btn-stop"
                    onclick={handleStopTimer}
                    title="Stop timer and save"
                  >
                    ⏹
                  </button>
                  {#if !timerStore.isRunning}
                    <button
                      class="btn-icon-compact btn-reset"
                      onclick={openResetModal}
                      title="Reset timer (discard time)"
                    >
                      ⟲
                    </button>
                  {/if}
                {:else}
                  <button
                    class="btn-icon-compact"
                    onclick={() => handleToggleTimer(task.id)}
                    title="Start timer"
                  >
                    ⏱
                  </button>
                  {#if task.total_time_seconds > 0}
                    <button
                      class="btn-icon-compact btn-reset"
                      onclick={openResetModal}
                      title="Reset timer (discard time)"
                    >
                      ⟲
                    </button>
                  {/if}
                {/if}
              </div>
            </div>
          {/each}

          {#if taskStore.tasks.length === 0}
            <div class="empty-state">
              <p class="text-secondary text-sm">No tasks yet</p>
              <p class="text-tertiary text-xs">Click "+ Task" to create your first task</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  {#if timerStore.active}
    <div class="timer-widget">
      <div class="timer-info">
        <div class="timer-task-name">{timerStore.active.task_title || "Task"}</div>
        <div class="timer-elapsed">
          <TimeDisplay seconds={Math.floor(timerStore.elapsed)} format="hms" />
        </div>
      </div>
      <div class="timer-controls">
        {#if timerStore.isRunning}
          <button class="btn btn-sm btn-secondary" onclick={() => timerStore.pause()}>
            Pause
          </button>
        {:else}
          <button class="btn btn-sm btn-primary" onclick={() => timerStore.resume()}>
            Resume
          </button>
        {/if}
        <button class="btn btn-sm btn-danger" onclick={handleStopTimer}>Stop</button>
      </div>
    </div>
  {/if}
</div>

<Modal open={uiStore.showProjectModal} title="New Project" onClose={() => uiStore.closeProjectModal()}>
  {#snippet children()}
    <form onsubmit={(e) => { e.preventDefault(); handleCreateProject(); }}>
      <div style="display: flex; flex-direction: column; gap: var(--spacing-md);">
        <div>
          <label for="project-name" class="text-sm text-secondary">Project Name</label>
          <input
            id="project-name"
            class="input"
            type="text"
            bind:value={projectName}
            placeholder="My Project"
            autofocus
          />
        </div>
        <div style="display: flex; gap: var(--spacing-sm); justify-content: flex-end;">
          <button type="button" class="btn btn-secondary" onclick={() => uiStore.closeProjectModal()}>
            Cancel
          </button>
          <button type="submit" class="btn btn-primary">Create</button>
        </div>
      </div>
    </form>
  {/snippet}
</Modal>

<Modal open={uiStore.showTaskModal} title="New Task" onClose={() => uiStore.closeTaskModal()}>
  {#snippet children()}
    <form onsubmit={(e) => { e.preventDefault(); handleCreateTask(); }}>
      <div style="display: flex; flex-direction: column; gap: var(--spacing-md);">
        <div>
          <label for="task-title" class="text-sm text-secondary">Task Title</label>
          <input
            id="task-title"
            class="input"
            type="text"
            bind:value={taskTitle}
            placeholder="Task title"
            autofocus
          />
        </div>
        <div style="display: flex; gap: var(--spacing-sm); justify-content: flex-end;">
          <button type="button" class="btn btn-secondary" onclick={() => uiStore.closeTaskModal()}>
            Cancel
          </button>
          <button type="submit" class="btn btn-primary">Create</button>
        </div>
      </div>
    </form>
  {/snippet}
</Modal>

<Modal open={showResetModal} title="⚠️ Reset Timer" onClose={() => showResetModal = false}>
  {#snippet children()}
    <div class="reset-modal-content">
      <div class="reset-warning">
        <div class="warning-icon">⚠️</div>
        <div class="warning-text">
          <p class="warning-title">Are you sure you want to reset the timer?</p>
          <p class="warning-description">
            All time tracking for this session will be permanently lost. This action cannot be undone.
          </p>
        </div>
      </div>
      <div class="reset-actions">
        <button type="button" class="btn btn-secondary" onclick={() => showResetModal = false}>
          Cancel
        </button>
        <button type="button" class="btn btn-warning" onclick={handleResetTimer}>
          Reset Timer
        </button>
      </div>
    </div>
  {/snippet}
</Modal>

<style>
  .main-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

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

  .projects-list,
  .tasks-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .project-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
    cursor: pointer;
    width: 100%;
    text-align: left;
  }

  .project-item:hover {
    background-color: var(--bg-hover);
  }

  .project-item.active {
    background-color: var(--accent-light);
    border-color: var(--accent);
  }

  .project-color {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .project-info {
    flex: 1;
    min-width: 0;
  }

  .project-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    width: 100%;
  }

  .project-name {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .project-time-badge {
    display: flex;
    align-items: center;
    padding: 2px 6px;
    background-color: var(--accent-light);
    color: var(--accent);
    border-radius: var(--radius-sm);
    font-size: 10px;
    font-weight: 600;
    font-family: var(--font-mono);
    flex-shrink: 0;
  }

  .project-item.active .project-time-badge {
    background-color: var(--accent);
    color: white;
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

  .task-checkbox {
    flex-shrink: 0;
    cursor: pointer;
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
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

  .time-icon {
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
    color: white;
    animation: pulse 2s ease-in-out infinite;
  }

  .inline-timer.paused {
    background-color: var(--warning);
    color: white;
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

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.85;
    }
  }

  @keyframes blink {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
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

  .empty-state p {
    margin-bottom: 0;
  }

  .timer-widget {
    flex-shrink: 0;
    padding: var(--spacing-md);
    background: linear-gradient(135deg, var(--success-light) 0%, var(--success-gradient-end) 100%);
    border-top: 2px solid var(--success);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
    box-shadow: var(--shadow-md);
  }

  .timer-info {
    flex: 1;
    min-width: 0;
  }

  .timer-task-name {
    font-weight: 600;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-primary);
    margin-bottom: 2px;
  }

  .timer-elapsed {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
    color: var(--success);
    letter-spacing: 0.5px;
  }

  .timer-controls {
    display: flex;
    gap: var(--spacing-sm);
  }

  .reset-modal-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .reset-warning {
    display: flex;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: linear-gradient(135deg, var(--warning-light) 0%, var(--warning-gradient-end) 100%);
    border: 2px solid var(--warning);
    border-radius: var(--radius-md);
  }

  .warning-icon {
    font-size: 32px;
    flex-shrink: 0;
    animation: warning-pulse 2s ease-in-out infinite;
  }

  @keyframes warning-pulse {
    0%, 100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.1);
    }
  }

  .warning-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .warning-title {
    font-weight: 600;
    font-size: 15px;
    color: var(--text-primary);
    margin: 0;
  }

  .warning-description {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .reset-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
  }
</style>
