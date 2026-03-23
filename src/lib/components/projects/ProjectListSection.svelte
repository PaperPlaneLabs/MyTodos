<script lang="ts">
  import { flip } from "svelte/animate";
  import { slide } from "svelte/transition";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";

  let {
    isDragging,
    draggedId,
    onKeySelect,
    onContextMenu,
    onPointerDown,
  }: {
    isDragging: boolean;
    draggedId: number | null;
    onKeySelect: (event: KeyboardEvent, id: number | null) => void;
    onContextMenu: (event: MouseEvent | PointerEvent, id: number) => void;
    onPointerDown: (event: PointerEvent, id: number, index: number) => void;
  } = $props();
</script>

<div class="projects-section">
  <div class="section-header">
    <h2>Projects</h2>
    <button class="btn btn-ghost btn-sm" onclick={() => uiStore.openProjectModal()}>
      + New
    </button>
  </div>

  <div class="projects-list" role="list">
    <div transition:slide={{ duration: 200 }}>
      <div
        class="project-item inbox-item"
        class:active={projectStore.selectedId === null}
        class:has-timer={timerStore.active && timerStore.currentProjectId === null}
        class:timer-running={timerStore.active &&
          timerStore.currentProjectId === null &&
          timerStore.isRunning}
        class:timer-paused={timerStore.active &&
          timerStore.currentProjectId === null &&
          !timerStore.isRunning}
        role="button"
        tabindex="0"
        onclick={() => projectStore.setSelected(null)}
        onkeydown={(event) => onKeySelect(event, null)}
      >
        <div class="project-info">
          <div class="project-header">
            <div class="project-name">Tasks</div>
            {#if timerStore.active && timerStore.currentProjectId === null}
              <div class="active-timer-dot" title="Active timer in this project"></div>
            {/if}
          </div>
        </div>
      </div>
    </div>

    {#each projectStore.projects as project, index (project.id)}
      <div
        animate:flip={{ duration: 300 }}
        transition:slide={{ duration: 200 }}
        class="draggable-wrapper"
        data-index={index}
        class:dragging={isDragging && draggedId === project.id}
      >
        <div
          class="project-item"
          class:active={projectStore.selectedId === project.id}
          class:has-timer={timerStore.active && timerStore.currentProjectId === project.id}
          class:timer-running={timerStore.active &&
            timerStore.currentProjectId === project.id &&
            timerStore.isRunning}
          class:timer-paused={timerStore.active &&
            timerStore.currentProjectId === project.id &&
            !timerStore.isRunning}
          style="border-left-color: {project.color};"
          role="button"
          tabindex="0"
          onclick={() => !isDragging && projectStore.setSelected(project.id)}
          onkeydown={(event) => onKeySelect(event, project.id)}
          oncontextmenu={(event) => onContextMenu(event, project.id)}
        >
          <div
            class="drag-handle"
            role="button"
            tabindex="0"
            aria-label="Drag to reorder"
            onpointerdown={(event) => onPointerDown(event, project.id, index)}
          >
            ⋮⋮
          </div>
          <div class="project-info">
            <div class="project-header">
              <div class="project-name">{project.name}</div>
              <div class="project-meta-right">
                {#if timerStore.active && timerStore.currentProjectId === project.id}
                  <div class="active-timer-dot" title="Active timer in this project"></div>
                {/if}
                {#if project.total_time_seconds > 0}
                  <div class="project-time-badge">
                    <TimeDisplay seconds={project.total_time_seconds} />
                  </div>
                {/if}
              </div>
            </div>
          </div>
        </div>
      </div>
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

  .projects-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .draggable-wrapper {
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

  .drag-handle {
    cursor: grab;
    color: var(--text-tertiary);
    opacity: 0;
    font-size: 14px;
    padding: 0 2px;
    transition: opacity 0.2s;
    flex-shrink: 0;
    margin-left: -4px;
    margin-right: 0;
  }

  .project-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-left-width: 3px;
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
    cursor: grab;
    width: 100%;
    text-align: left;
    position: relative;
    user-select: none;
  }

  .project-item:active {
    cursor: grabbing;
  }

  .project-item:hover {
    background-color: var(--bg-hover);
  }

  .project-item:hover .drag-handle {
    opacity: 1;
  }

  .project-item.active {
    background-color: var(--accent-light);
    border-color: var(--accent);
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

  .project-meta-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex-shrink: 0;
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

  .active-timer-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--success);
    box-shadow: 0 0 10px var(--success-glow);
  }

  .timer-running .active-timer-dot {
    background-color: var(--success);
    box-shadow: 0 0 12px var(--success-glow);
    animation: dot-pulse 1.5s ease-in-out infinite;
  }

  .timer-paused .active-timer-dot {
    background-color: var(--warning);
    box-shadow: 0 0 8px var(--warning-glow);
    animation: none;
  }

  .project-item.timer-running {
    border-color: var(--success-light);
    background-color: var(--success-glow);
  }

  .project-item.timer-paused {
    border-color: var(--warning-light);
    background-color: var(--warning-glow);
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

  @keyframes dot-pulse {
    0%,
    100% {
      transform: scale(1);
      opacity: 1;
    }

    50% {
      transform: scale(1.3);
      opacity: 0.7;
    }
  }
</style>
