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
              <div class="project-name">{project.name}</div>
              {#if project.total_time_seconds > 0}
                <div class="project-time text-xs text-secondary">
                  <TimeDisplay seconds={project.total_time_seconds} />
                </div>
              {/if}
            </div>
          </button>
        {/each}

        {#if projectStore.projects.length === 0}
          <div class="empty-state">
            <p class="text-secondary text-sm">No projects yet</p>
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
            <div class="task-item">
              <input
                type="checkbox"
                checked={task.completed}
                onchange={() => taskStore.toggleCompletion(task.id)}
                class="task-checkbox"
              />
              <div class="task-content">
                <div class="task-title" class:completed={task.completed}>{task.title}</div>
                {#if task.total_time_seconds > 0}
                  <div class="task-time text-xs text-secondary">
                    <TimeDisplay seconds={task.total_time_seconds} />
                  </div>
                {/if}
              </div>
              <button
                class="btn-icon"
                onclick={() => handleToggleTimer(task.id)}
                title={timerStore.active?.task_id === task.id
                  ? timerStore.isRunning
                    ? "Pause"
                    : "Resume"
                  : "Start timer"}
              >
                {#if timerStore.active && timerStore.active.task_id === task.id}
                  {#if timerStore.isRunning}
                    ⏸
                  {:else}
                    ▶
                  {/if}
                {:else}
                  ⏱
                {/if}
              </button>
            </div>
          {/each}

          {#if taskStore.tasks.length === 0}
            <div class="empty-state">
              <p class="text-secondary text-sm">No tasks yet</p>
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
  }

  .section-header h2 {
    font-size: 16px;
    font-weight: 600;
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

  .project-name {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .task-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
  }

  .task-item:hover {
    background-color: var(--bg-hover);
  }

  .task-checkbox {
    flex-shrink: 0;
    cursor: pointer;
    width: 18px;
    height: 18px;
  }

  .task-content {
    flex: 1;
    min-width: 0;
  }

  .task-title {
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .task-title.completed {
    text-decoration: line-through;
    color: var(--text-secondary);
  }

  .btn-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
    font-size: 14px;
    flex-shrink: 0;
  }

  .btn-icon:hover {
    background-color: var(--accent-light);
    color: var(--accent);
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    align-items: center;
  }

  .timer-widget {
    flex-shrink: 0;
    padding: var(--spacing-md);
    background-color: var(--success-light);
    border-top: 2px solid var(--success);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
  }

  .timer-info {
    flex: 1;
    min-width: 0;
  }

  .timer-task-name {
    font-weight: 600;
    font-size: 14px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .timer-elapsed {
    font-family: var(--font-mono);
    font-size: 18px;
    font-weight: 700;
    color: var(--success);
  }

  .timer-controls {
    display: flex;
    gap: var(--spacing-sm);
  }
</style>
