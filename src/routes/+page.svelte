<script lang="ts">
  import { onMount } from "svelte";
  import { fade, slide, fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import "$lib/styles/global.css";
  import AppHeader from "$lib/components/layout/AppHeader.svelte";
  import CollapseHandle from "$lib/components/layout/CollapseHandle.svelte";
  import StatsView from "$lib/components/stats/StatsView.svelte";
  import SettingsView from "$lib/components/settings/SettingsView.svelte";
  import CalendarTabView from "$lib/components/calendar/CalendarTabView.svelte";
  import Modal from "$lib/components/common/Modal.svelte";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import UpdateNotification from "$lib/components/common/UpdateNotification.svelte";
  import BreakView from "$lib/components/common/BreakView.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { googleCalendarStore } from "$lib/stores/google-calendar.svelte";

  // ── Multi-window: check if this is the break reminder window ──────────
  const isBreakWindow =
    window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label === "break";
  let projectName = $state("");
  let taskTitle = $state("");
  let taskDeadline = $state<string | null>(null);
  let taskTime = $state("");
  let isCalendarPresetDeadline = $derived(
    uiStore.showTaskModal &&
      !uiStore.editingTaskId &&
      !!uiStore.newTaskDeadline,
  );
  let showResetModal = $state(false);
  let taskToReset = $state<number | null>(null);
  // Deletion State
  let showDeleteModal = $state(false);
  let itemToDelete = $state<{ type: "project" | "task"; id: number } | null>(
    null,
  );

  // Pointer-based Drag and Drop State
  let isDragging = $state(false);
  let dragType = $state<"project" | "task" | null>(null);
  let draggedId = $state<number | null>(null);
  let startIndex = $state<number | null>(null);
  let currentIndex = $state<number | null>(null);
  let pointerId = $state<number | null>(null);
  let startY = $state(0);
  let startX = $state(0);
  let hasMovedThreshold = $state(false);

  // Long-press State
  let longPressTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  let lastEditingProjectId = $state<number | null>(null);

  function formatDeadline(deadline: string | null | undefined): string {
    if (!deadline) return "";
    const hasTime = deadline.includes("T");
    const date = new Date(deadline);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);

    let dateStr = "";
    if (date.toDateString() === today.toDateString()) {
      dateStr = "Due Today";
    } else if (date.toDateString() === tomorrow.toDateString()) {
      dateStr = "Due Tomorrow";
    } else {
      dateStr = date.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      });
    }

    if (hasTime) {
      const timeStr = date.toLocaleTimeString("en-US", {
        hour: "numeric",
        minute: "2-digit",
      });
      return `${dateStr} ${timeStr}`;
    }
    return dateStr;
  }

  function formatPresetDeadline(deadline: string | null): string {
    if (!deadline) return "";
    const date = new Date(`${deadline}T00:00:00`);
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  function isOverdue(
    deadline: string | null | undefined,
    completed: boolean,
  ): boolean {
    if (!deadline || completed) return false;
    return new Date(deadline) < new Date();
  }

  $effect(() => {
    // Sync projectName ONLY when the modal opens or a different project is selected
    if (
      uiStore.showProjectModal &&
      uiStore.editingProjectId !== lastEditingProjectId
    ) {
      lastEditingProjectId = uiStore.editingProjectId;
      if (uiStore.editingProjectId) {
        const project = projectStore.projects.find(
          (p) => p.id === uiStore.editingProjectId,
        );
        if (project) projectName = project.name;
      } else {
        projectName = "";
      }
    } else if (!uiStore.showProjectModal) {
      lastEditingProjectId = null;
    }
  });

  let lastTaskModalKey = $state<string | null>(null);

  $effect(() => {
    // Sync task fields only when the modal context changes.
    if (uiStore.showTaskModal) {
      const modalKey = `${uiStore.editingTaskId ?? "new"}:${uiStore.newTaskDeadline ?? ""}`;
      if (modalKey === lastTaskModalKey) {
        return;
      }
      lastTaskModalKey = modalKey;

      if (uiStore.editingTaskId) {
        const task = taskStore.tasks.find(
          (t) => t.id === uiStore.editingTaskId,
        );
        if (task) {
          taskTitle = task.title;
          if (task.deadline) {
            const [date, time] = task.deadline.split("T");
            taskDeadline = date;
            taskTime = time ? time.substring(0, 5) : "";
          } else {
            taskDeadline = null;
            taskTime = "";
          }
        }
      } else {
        taskTitle = "";
        taskDeadline = uiStore.newTaskDeadline;
        taskTime = "";
      }
    } else {
      lastTaskModalKey = null;
      taskTitle = "";
      taskDeadline = null;
      taskTime = "";
    }
  });

  onMount(async () => {
    uiStore.initTheme();
    timerStore.initBreakReminders();
    await projectStore.loadAll();
    await timerStore.loadActive();
    googleCalendarStore.init();
  });

  $effect(() => {
    taskStore.loadByProject(projectStore.selectedId);
  });

  async function handleCreateProject() {
    if (!projectName.trim()) return;
    await projectStore.create(projectName);
    projectName = "";
    uiStore.closeProjectModal();
  }

  async function handleCreateTask() {
    if (!taskTitle.trim()) return;
    try {
      const task = await taskStore.createTask(
        projectStore.selectedId,
        null,
        taskTitle,
      );
      if (taskDeadline) {
        const fullDeadline = taskTime
          ? `${taskDeadline}T${taskTime}`
          : taskDeadline;
        await taskStore.updateDeadline(task.id, fullDeadline);
      }
      taskTitle = "";
      taskDeadline = null;
      taskTime = "";
      uiStore.closeTaskModal();
    } catch (e) {
      console.error("Error creating task in UI:", e);
    }
  }

  async function handleEditTask() {
    if (!taskTitle.trim()) return;
    if (!uiStore.editingTaskId) return;

    try {
      await taskStore.updateTask(uiStore.editingTaskId, taskTitle);
      const fullDeadline = taskDeadline
        ? taskTime
          ? `${taskDeadline}T${taskTime}`
          : taskDeadline
        : null;
      await taskStore.updateDeadline(uiStore.editingTaskId, fullDeadline);
      uiStore.closeTaskModal();
    } catch (e) {
      console.error("Error updating task:", e);
      // Could show a toast/alert to user here
    }
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
  }

  function openResetModal(taskId: number) {
    taskToReset = taskId;
    showResetModal = true;
  }

  async function handleResetTimer() {
    if (taskToReset === null) return;

    if (timerStore.active && timerStore.active.task_id === taskToReset) {
      await timerStore.reset();
    } else {
      await taskStore.resetTaskTime(taskToReset);
      await projectStore.loadAll();
    }

    showResetModal = false;
    taskToReset = null;
  }

  // Deletion Handlers
  function confirmDelete(type: "project" | "task", id: number) {
    itemToDelete = { type, id };
    showDeleteModal = true;
  }

  async function handleDelete() {
    if (!itemToDelete) return;

    try {
      if (itemToDelete.type === "project") {
        await projectStore.delete(itemToDelete.id);
      } else {
        // If deleting task that has active timer, stop timer first
        if (
          timerStore.active &&
          timerStore.active.task_id === itemToDelete.id
        ) {
          await timerStore.stop();
        }
        await taskStore.deleteTask(itemToDelete.id);
      }
    } catch (e) {
      console.error(`Failed to delete ${itemToDelete.type}:`, e);
    }

    showDeleteModal = false;
    itemToDelete = null;
  }

  // Context Menu Handler
  function handleContextMenu(
    e: MouseEvent | PointerEvent,
    type: "project" | "task",
    id: number,
  ) {
    e.preventDefault();
    uiStore.openContextMenu(e.clientX, e.clientY, type, id);
  }

  // Handlers for Pointer Events
  function handlePointerDown(
    e: PointerEvent,
    type: "project" | "task",
    id: number,
    index: number,
  ) {
    if (e.button === 2) return; // Ignore right click for drag

    pointerId = e.pointerId;
    dragType = type;
    draggedId = id;
    startIndex = index;
    currentIndex = index;
    startY = e.clientY;
    startX = e.clientX;
    hasMovedThreshold = false;

    // Start long press timer (for mobile)
    if (longPressTimer) clearTimeout(longPressTimer);
    longPressTimer = setTimeout(() => {
      if (!hasMovedThreshold) {
        uiStore.openContextMenu(startX, startY, type, id);
        cancelDrag();
      }
    }, 600);
  }

  function handlePointerMove(e: PointerEvent) {
    if (pointerId !== e.pointerId || draggedId === null) return;

    // Check threshold
    if (!hasMovedThreshold) {
      if (
        Math.abs(e.clientY - startY) > 5 ||
        Math.abs(e.clientX - startX) > 5
      ) {
        hasMovedThreshold = true;
        isDragging = true;
        if (longPressTimer) {
          clearTimeout(longPressTimer);
          longPressTimer = null;
        }
      } else {
        return;
      }
    }

    // Safety check: is the button still down?
    if (e.buttons !== 1) {
      handlePointerUp(e);
      return;
    }

    const elem = document.elementFromPoint(e.clientX, e.clientY);
    const wrapper = elem?.closest(".draggable-wrapper, .task-item-wrapper");

    if (wrapper) {
      const type = wrapper.classList.contains("draggable-wrapper")
        ? "project"
        : "task";
      if (type === dragType) {
        const newIndex = parseInt(wrapper.getAttribute("data-index") || "-1");
        if (newIndex !== -1 && newIndex !== currentIndex) {
          if (dragType === "project") {
            projectStore.reorderLocal(currentIndex!, newIndex);
          } else {
            taskStore.reorderLocal(currentIndex!, newIndex);
          }
          currentIndex = newIndex;
        }
      }
    }
  }

  async function handlePointerUp(e: PointerEvent) {
    if (pointerId !== e.pointerId) return;

    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }

    if (isDragging) {
      try {
        if (dragType === "project") {
          const ids = projectStore.projects.map((p) => p.id);
          await projectStore.reorder(ids);
        } else if (dragType === "task") {
          const ids = taskStore.tasks.map((t) => t.id);
          await taskStore.reorder(ids);
        }
      } catch (err) {
        console.error("Failed to save order:", err);
      }
    }

    cancelDrag();
  }

  function cancelDrag() {
    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
    isDragging = false;
    dragType = null;
    draggedId = null;
    startIndex = null;
    currentIndex = null;
    pointerId = null;
    hasMovedThreshold = false;
  }

  function handleKeySelect(e: KeyboardEvent, id: number | null) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      projectStore.setSelected(id);
    }
  }

  // Context Menu Items
  let contextMenuItems = $derived.by(() => {
    if (!uiStore.contextMenuType || uiStore.contextMenuId === null) return [];

    const id = uiStore.contextMenuId;
    const type = uiStore.contextMenuType;

    if (type === "project") {
      return [
        {
          label: "Edit Project",
          icon: "✏️",
          onClick: () => uiStore.openProjectModal(id),
        },
        {
          label: "Delete Project",
          icon: "🗑️",
          danger: true,
          onClick: () => confirmDelete("project", id),
        },
      ];
    } else {
      return [
        {
          label: "Edit Task",
          icon: "✏️",
          onClick: () => uiStore.openTaskModal(id),
        },
        {
          label: "Delete Task",
          icon: "🗑️",
          danger: true,
          onClick: () => confirmDelete("task", id),
        },
      ];
    }
  });
</script>

<svelte:window
  onpointermove={!isBreakWindow ? handlePointerMove : undefined}
  onpointerup={!isBreakWindow ? handlePointerUp : undefined}
  onpointercancel={!isBreakWindow ? cancelDrag : undefined}
  onkeydown={!isBreakWindow
    ? (e) => {
        if (e.key === "Escape") cancelDrag();
      }
    : undefined}
  onclick={!isBreakWindow ? () => uiStore.closeContextMenu() : undefined}
/>

{#if isBreakWindow}
  <BreakView />
{:else}
  <div class="app-container" class:app-collapsed={uiStore.isCollapsed}>
    <CollapseHandle />

    {#if !uiStore.isCollapsed}
      <AppHeader />

      {#if uiStore.showCalendarView}
        <CalendarTabView />
      {:else if uiStore.showStatsView}
        <StatsView />
      {:else if uiStore.showSettingsView}
        <SettingsView />
      {:else}
        <div class="main-content">
          <div class="projects-section">
            <div class="section-header">
              <h2>Projects</h2>
              <button
                class="btn btn-ghost btn-sm"
                onclick={() => uiStore.openProjectModal()}
              >
                + New
              </button>
            </div>

            <div class="projects-list" role="list">
              <div transition:slide={{ duration: 200 }}>
                <div
                  class="project-item inbox-item"
                  class:active={projectStore.selectedId === null}
                  class:has-timer={timerStore.active &&
                    timerStore.currentProjectId === null}
                  class:timer-running={timerStore.active &&
                    timerStore.currentProjectId === null &&
                    timerStore.isRunning}
                  class:timer-paused={timerStore.active &&
                    timerStore.currentProjectId === null &&
                    !timerStore.isRunning}
                  role="button"
                  tabindex="0"
                  onclick={() => projectStore.setSelected(null)}
                  onkeydown={(e) => handleKeySelect(e, null)}
                >
                  <div
                    class="project-color"
                    style="background-color: var(--text-tertiary)"
                  ></div>
                  <div class="project-info">
                    <div class="project-header">
                      <div class="project-name">Tasks</div>
                      {#if timerStore.active && timerStore.currentProjectId === null}
                        <div
                          class="active-timer-dot"
                          title="Active timer in this project"
                        ></div>
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
                    class:has-timer={timerStore.active &&
                      timerStore.currentProjectId === project.id}
                    class:timer-running={timerStore.active &&
                      timerStore.currentProjectId === project.id &&
                      timerStore.isRunning}
                    class:timer-paused={timerStore.active &&
                      timerStore.currentProjectId === project.id &&
                      !timerStore.isRunning}
                    role="button"
                    tabindex="0"
                    onclick={() =>
                      !isDragging && projectStore.setSelected(project.id)}
                    onkeydown={(e) => handleKeySelect(e, project.id)}
                    oncontextmenu={(e) =>
                      handleContextMenu(e, "project", project.id)}
                  >
                    <div
                      class="project-color"
                      style="background-color: {project.color}"
                    ></div>
                    <div class="project-info">
                      <div class="project-header">
                        <div class="project-name">{project.name}</div>
                        <div class="project-meta-right">
                          {#if timerStore.active && timerStore.currentProjectId === project.id}
                            <div
                              class="active-timer-dot"
                              title="Active timer in this project"
                            ></div>
                          {/if}
                          {#if project.total_time_seconds > 0}
                            <div class="project-time-badge">
                              <TimeDisplay
                                seconds={project.total_time_seconds}
                              />
                            </div>
                          {/if}
                        </div>
                      </div>
                    </div>
                    <div
                      class="drag-handle"
                      aria-label="Drag to reorder"
                      onpointerdown={(e) =>
                        handlePointerDown(e, "project", project.id, index)}
                    >
                      ⋮⋮
                    </div>
                  </div>
                </div>
              {/each}

              {#if projectStore.projects.length === 0}
                <div class="empty-state">
                  <p class="text-secondary text-sm">No projects yet</p>
                  <p class="text-tertiary text-xs">
                    Click "+ New" to create your first project
                  </p>
                  <button
                    class="btn btn-primary btn-sm"
                    onclick={() => uiStore.openProjectModal()}
                  >
                    Create Project
                  </button>
                </div>
              {/if}
            </div>
          </div>

          {#if projectStore.selectedId !== undefined}
            <div class="tasks-section">
              <div class="section-header">
                <h2>{projectStore.selected?.name || "Tasks"}</h2>
                <button
                  class="btn btn-ghost btn-sm"
                  onclick={() => uiStore.openTaskModal()}
                >
                  + Task
                </button>
              </div>

              <div class="tasks-list" role="list">
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
                      class:task-timer-active={timerStore.active?.task_id ===
                        task.id && timerStore.isRunning}
                      class:task-timer-paused={timerStore.active?.task_id ===
                        task.id && !timerStore.isRunning}
                      oncontextmenu={(e) =>
                        handleContextMenu(e, "task", task.id)}
                      onpointerdown={(e) => {
                        // If it's the checkbox or controls, don't trigger long press on the whole item
                        const target = e.target as HTMLElement;
                        if (
                          target.closest(".checkbox-container") ||
                          target.closest(".task-controls")
                        )
                          return;
                        handlePointerDown(e, "task", task.id, index);
                      }}
                    >
                      <div
                        class="drag-handle-task"
                        onpointerdown={(e) => {
                          e.stopPropagation();
                          handlePointerDown(e, "task", task.id, index);
                        }}
                      >
                        ⋮⋮
                      </div>
                      <label class="checkbox-container">
                        <input
                          type="checkbox"
                          checked={task.completed}
                          onchange={() => taskStore.toggleCompletion(task.id)}
                          class="task-checkbox-hidden"
                        />
                        <span class="checkbox-custom"></span>
                      </label>
                      <div class="task-content">
                        <div
                          class="task-title"
                          class:completed={task.completed}
                        >
                          {task.title}
                        </div>
                        <div class="task-meta">
                          {#if task.deadline}
                            <div
                              class="task-deadline text-xs"
                              class:overdue={isOverdue(
                                task.deadline,
                                task.completed,
                              )}
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
                              <TimeDisplay
                                seconds={Math.floor(timerStore.elapsed)}
                                format="short"
                              />
                            </div>
                          {/if}
                        </div>
                      </div>
                      <div class="task-controls">
                        {#if timerStore.active && timerStore.active.task_id === task.id}
                          <button
                            class="btn-icon-compact"
                            onclick={() =>
                              timerStore.isRunning
                                ? timerStore.pause()
                                : timerStore.resume()}
                            title={timerStore.isRunning
                              ? "Pause timer"
                              : "Resume timer"}
                          >
                            {#if timerStore.isRunning}⏸{:else}▶{/if}
                          </button>
                          <button
                            class="btn-icon-compact btn-stop"
                            onclick={handleStopTimer}
                            title="Stop timer and save">⏹</button
                          >
                          {#if !timerStore.isRunning}
                            <button
                              class="btn-icon-compact btn-reset"
                              onclick={() => openResetModal(task.id)}
                              title="Reset timer (discard time)">⟲</button
                            >
                          {/if}
                        {:else if task.project_id}
                          <button
                            class="btn-icon-compact"
                            onclick={() => handleToggleTimer(task.id)}
                            title="Start timer">⏱</button
                          >
                          {#if task.total_time_seconds > 0}
                            <button
                              class="btn-icon-compact btn-reset"
                              onclick={() => openResetModal(task.id)}
                              title="Reset all time for this task">⟲</button
                            >
                          {/if}
                        {/if}
                      </div>
                    </div>
                  </div>
                {/each}

                {#if taskStore.completedTasks.length > 0}
                  <button
                    class="completed-separator"
                    transition:slide={{ duration: 200 }}
                    onclick={() => uiStore.toggleCompletedTasks()}
                    aria-expanded={!uiStore.completedTasksCollapsed}
                  >
                    <span
                      class="completed-caret"
                      class:expanded={!uiStore.completedTasksCollapsed}
                    >
                      <svg
                        width="10"
                        height="10"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="3"
                        stroke-linecap="round"
                        stroke-linejoin="round"><path d="m9 18 6-6-6-6" /></svg
                      >
                    </span>
                    <span class="completed-label">Completed</span>
                    <div class="completed-line"></div>
                    <span class="completed-count"
                      >{taskStore.completedTasks.length}</span
                    >
                  </button>

                  {#if !uiStore.completedTasksCollapsed}
                    <div
                      class="completed-tasks-container"
                      transition:slide={{ duration: 250 }}
                    >
                      {#each taskStore.completedTasks as task, index (task.id)}
                        {@const globalIndex =
                          index + taskStore.activeTasks.length}
                        <div
                          animate:flip={{ duration: 300 }}
                          class="task-item-wrapper"
                          data-index={globalIndex}
                          class:dragging={isDragging && draggedId === task.id}
                        >
                          <div
                            class="task-item completed-task-item"
                            oncontextmenu={(e) =>
                              handleContextMenu(e, "task", task.id)}
                            onpointerdown={(e) => {
                              const target = e.target as HTMLElement;
                              if (
                                target.closest(".checkbox-container") ||
                                target.closest(".task-controls")
                              )
                                return;
                              handlePointerDown(
                                e,
                                "task",
                                task.id,
                                globalIndex,
                              );
                            }}
                          >
                            <div
                              class="drag-handle-task"
                              onpointerdown={(e) => {
                                e.stopPropagation();
                                handlePointerDown(
                                  e,
                                  "task",
                                  task.id,
                                  globalIndex,
                                );
                              }}
                            >
                              ⋮⋮
                            </div>
                            <label class="checkbox-container">
                              <input
                                type="checkbox"
                                checked={task.completed}
                                onchange={() =>
                                  taskStore.toggleCompletion(task.id)}
                                class="task-checkbox-hidden"
                              />
                              <span class="checkbox-custom"></span>
                            </label>
                            <div class="task-content">
                              <div
                                class="task-title"
                                class:completed={task.completed}
                              >
                                {task.title}
                              </div>
                              <div class="task-meta">
                                {#if task.deadline}
                                  <div
                                    class="task-deadline text-xs"
                                    class:overdue={isOverdue(
                                      task.deadline,
                                      task.completed,
                                    )}
                                  >
                                    <span class="deadline-icon">📅</span>
                                    {formatDeadline(task.deadline)}
                                  </div>
                                {/if}
                                {#if task.total_time_seconds > 0 && task.project_id}
                                  <div class="task-time text-xs">
                                    <span class="time-icon">⏱</span>
                                    <TimeDisplay
                                      seconds={task.total_time_seconds}
                                    />
                                  </div>
                                {/if}
                              </div>
                            </div>
                            <div class="task-controls">
                              {#if task.project_id && task.total_time_seconds > 0}
                                <button
                                  class="btn-icon-compact btn-reset"
                                  onclick={() => openResetModal(task.id)}
                                  title="Reset all time for this task">⟲</button
                                >
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
                    <p class="text-tertiary text-xs">
                      Click "+ Task" to create your first task
                    </p>
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if timerStore.active}
        <div class="timer-widget" transition:fly={{ y: 50, duration: 300 }}>
          {#if timerStore.isAutoPaused}
            <div class="auto-pause-banner">
              <span class="icon">⏸️</span>
              <span>
                Timer auto-paused due to
                {#if timerStore.autoPausedReason === "SystemSleep"}
                  system sleep
                {:else if timerStore.autoPausedReason === "ScreenLock"}
                  screen lock
                {:else}
                  shutdown
                {/if}
              </span>
            </div>
          {/if}
          <div class="timer-content">
            <div class="timer-info">
              <div class="timer-task-name">
                {timerStore.active.task_title || "Task"}
              </div>
              <div class="timer-elapsed">
                <TimeDisplay
                  seconds={Math.floor(timerStore.elapsed)}
                  format="hms"
                />
              </div>
            </div>
            <div class="timer-controls">
              {#if timerStore.isRunning}
                <button
                  class="btn btn-sm btn-secondary"
                  onclick={() => timerStore.pause()}>Pause</button
                >
              {:else}
                <button
                  class="btn btn-sm btn-primary"
                  onclick={() => timerStore.resume()}>Resume</button
                >
              {/if}
              <button class="btn btn-sm btn-danger" onclick={handleStopTimer}
                >Stop</button
              >
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>

  <ContextMenu items={contextMenuItems} />

  <Modal
    open={uiStore.showProjectModal}
    title={uiStore.editingProjectId ? "Edit Project" : "New Project"}
    onClose={() => uiStore.closeProjectModal()}
  >
    {#snippet children()}
      <form
        onsubmit={(e) => {
          e.preventDefault();
          if (uiStore.editingProjectId) {
            projectStore.update(uiStore.editingProjectId, projectName);
            uiStore.closeProjectModal();
          } else {
            handleCreateProject();
          }
        }}
      >
        <div
          style="display: flex; flex-direction: column; gap: var(--spacing-md);"
        >
          <div>
            <label for="project-name" class="text-sm text-secondary"
              >Project Name</label
            >
            <input
              id="project-name"
              class="input"
              type="text"
              bind:value={projectName}
              placeholder="My Project"
              autofocus
            />
          </div>
          <div
            style="display: flex; gap: var(--spacing-sm); justify-content: flex-end;"
          >
            <button
              type="button"
              class="btn btn-secondary"
              onclick={() => uiStore.closeProjectModal()}>Cancel</button
            >
            <button type="submit" class="btn btn-primary"
              >{uiStore.editingProjectId ? "Save" : "Create"}</button
            >
          </div>
        </div>
      </form>
    {/snippet}
  </Modal>

  <Modal
    open={uiStore.showTaskModal}
    title={uiStore.editingTaskId ? "Edit Task" : "New Task"}
    onClose={() => uiStore.closeTaskModal()}
  >
    {#snippet children()}
      <form
        onsubmit={(e) => {
          e.preventDefault();
          if (uiStore.editingTaskId) {
            handleEditTask();
          } else {
            handleCreateTask();
          }
        }}
      >
        <div
          style="display: flex; flex-direction: column; gap: var(--spacing-md);"
        >
          <div>
            <label for="task-title" class="text-sm text-secondary"
              >Task Title</label
            >
            <input
              id="task-title"
              class="input"
              type="text"
              bind:value={taskTitle}
              placeholder="Task title"
              autofocus
            />
          </div>

          <div>
            {#if isCalendarPresetDeadline}
              <div class="text-sm text-secondary">Deadline</div>
              <div class="deadline-input">
                <div class="deadline-fixed">
                  <span>{formatPresetDeadline(taskDeadline)}</span>
                </div>
                <input
                  type="time"
                  class="input"
                  bind:value={taskTime}
                  step="300"
                  style="width: 110px;"
                />
              </div>
            {:else}
              <label for="task-deadline" class="text-sm text-secondary"
                >Deadline (optional)</label
              >
              <div class="deadline-input">
                <input
                  id="task-deadline"
                  class="input"
                  type="date"
                  bind:value={taskDeadline}
                  placeholder="No deadline"
                />
                <input
                  type="time"
                  class="input"
                  bind:value={taskTime}
                  step="300"
                  style="width: 110px;"
                  disabled={!taskDeadline}
                />
                {#if taskDeadline}
                  <button
                    type="button"
                    class="btn btn-ghost"
                    onclick={() => {
                      taskDeadline = null;
                      taskTime = "";
                    }}
                  >
                    ✕
                  </button>
                {/if}
              </div>
            {/if}
          </div>

          <div
            style="display: flex; gap: var(--spacing-sm); justify-content: flex-end;"
          >
            <button
              type="button"
              class="btn btn-secondary"
              onclick={() => uiStore.closeTaskModal()}>Cancel</button
            >
            <button type="submit" class="btn btn-primary"
              >{uiStore.editingTaskId ? "Save" : "Create"}</button
            >
          </div>
        </div>
      </form>
    {/snippet}
  </Modal>

  <Modal
    open={showResetModal}
    title="⚠️ Reset Timer"
    onClose={() => (showResetModal = false)}
  >
    {#snippet children()}
      <div class="reset-modal-content">
        <div class="reset-warning">
          <div class="warning-icon">⚠️</div>
          <div class="warning-text">
            <p class="warning-title">
              Are you sure you want to reset the timer?
            </p>
            <p class="warning-description">
              All time tracking for this session will be permanently lost.
            </p>
          </div>
        </div>
        <div class="reset-actions">
          <button
            type="button"
            class="btn btn-secondary"
            onclick={() => (showResetModal = false)}>Cancel</button
          >
          <button
            type="button"
            class="btn btn-warning"
            onclick={handleResetTimer}>Reset Timer</button
          >
        </div>
      </div>
    {/snippet}
  </Modal>

  <Modal
    open={showDeleteModal}
    title="⚠️ Delete {itemToDelete?.type === 'project' ? 'Project' : 'Task'}"
    onClose={() => (showDeleteModal = false)}
  >
    {#snippet children()}
      <div class="reset-modal-content">
        <div
          class="reset-warning"
          style="background: linear-gradient(135deg, var(--danger-light) 0%, var(--danger-glow) 100%); border-color: var(--danger);"
        >
          <div class="warning-icon">🗑️</div>
          <div class="warning-text">
            <p class="warning-title">Delete this {itemToDelete?.type}?</p>
            <p class="warning-description">
              This action cannot be undone. All associated data will be lost.
            </p>
          </div>
        </div>
        <div class="reset-actions">
          <button
            type="button"
            class="btn btn-secondary"
            onclick={() => (showDeleteModal = false)}>Cancel</button
          >
          <button type="button" class="btn btn-danger" onclick={handleDelete}
            >Delete</button
          >
        </div>
      </div>
    {/snippet}
  </Modal>

  <UpdateNotification />
{/if}

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: var(--bg-primary);
    overflow: hidden;
  }

  .app-container.app-collapsed {
    background-color: transparent;
  }

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

  /* Date and Time Input Theme Support */
  input[type="date"],
  input[type="time"] {
    appearance: none;
    -webkit-appearance: none;
    font-family: var(--font-mono);
  }

  /* Force dark color scheme for native pickers in dark themes */
  :global([data-theme="dark"]) input[type="date"],
  :global([data-theme="dark"]) input[type="time"],
  :global([data-theme="retro"]) input[type="date"],
  :global([data-theme="retro"]) input[type="time"],
  :global([data-theme="ocean"]) input[type="date"],
  :global([data-theme="ocean"]) input[type="time"],
  :global([data-theme="nord"]) input[type="date"],
  :global([data-theme="nord"]) input[type="time"],
  :global([data-theme="minecraft"]) input[type="date"],
  :global([data-theme="minecraft"]) input[type="time"] {
    color-scheme: dark;
  }

  /* Style the picker indicator icon */
  input[type="date"]::-webkit-calendar-picker-indicator,
  input[type="time"]::-webkit-calendar-picker-indicator {
    cursor: pointer;
    opacity: 0.5;
    transition: all 0.2s;
    filter: invert(0); /* Default for light themes */
  }

  /* Invert icon for dark themes if color-scheme doesn't handle it fully */
  :global([data-theme="dark"])
    input[type="date"]::-webkit-calendar-picker-indicator,
  :global([data-theme="dark"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="retro"])
    input[type="date"]::-webkit-calendar-picker-indicator,
  :global([data-theme="retro"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="ocean"])
    input[type="date"]::-webkit-calendar-picker-indicator,
  :global([data-theme="ocean"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="nord"])
    input[type="date"]::-webkit-calendar-picker-indicator,
  :global([data-theme="nord"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="minecraft"])
    input[type="date"]::-webkit-calendar-picker-indicator,
  :global([data-theme="minecraft"])
    input[type="time"]::-webkit-calendar-picker-indicator {
    filter: invert(1);
    opacity: 0.7;
  }

  input[type="date"]::-webkit-calendar-picker-indicator:hover,
  input[type="time"]::-webkit-calendar-picker-indicator:hover {
    opacity: 1;
    transform: scale(1.1);
  }

  .draggable-wrapper,
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

  .drag-handle,
  .drag-handle-task {
    cursor: grab;
    color: var(--text-tertiary);
    opacity: 0;
    font-size: 14px;
    padding: 0 4px;
    margin-right: -4px;
    transition: opacity 0.2s;
  }

  .project-item:hover .drag-handle,
  .task-item:hover .drag-handle-task {
    opacity: 1;
  }

  .drag-handle-task {
    margin-right: 4px;
    margin-left: -4px;
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

  .project-item.active {
    background-color: var(--accent-light);
    border-color: var(--accent);
  }

  .project-color {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex-shrink: 0;
    transition: transform var(--transition-fast);
  }

  .project-item:hover .project-color {
    transform: scale(1.3);
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
    border: solid white;
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

  .deadline-input {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .deadline-fixed {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: var(--text-sm);
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

  .timer-widget {
    flex-shrink: 0;
    padding: var(--spacing-md);
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-top: 3px solid var(--success);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    display: flex;
    flex-direction: column;
    gap: 0;
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.08);
    margin: 0 var(--spacing-sm);
    z-index: 100;
  }

  :global([data-theme="dark"]) .timer-widget {
    background: var(--bg-secondary);
    box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.4);
  }

  .auto-pause-banner {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    background-color: var(--warning-light);
    border: 1px solid var(--warning);
    border-radius: var(--radius-md);
    font-size: 12px;
    color: var(--text-primary);
    margin-bottom: var(--spacing-sm);
  }

  .auto-pause-banner .icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  :global([data-theme="dark"]) .auto-pause-banner {
    background-color: rgba(251, 191, 36, 0.15);
    border-color: var(--warning);
  }

  .timer-content {
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
    background: linear-gradient(
      135deg,
      var(--warning-light) 0%,
      var(--warning-gradient-end) 100%
    );
    border: 2px solid var(--warning);
    border-radius: var(--radius-md);
  }

  .warning-icon {
    font-size: 32px;
    flex-shrink: 0;
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
