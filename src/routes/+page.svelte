<script lang="ts">
  import { onMount } from "svelte";
  import "$lib/styles/global.css";
  import AppHeader from "$lib/components/layout/AppHeader.svelte";
  import CollapseHandle from "$lib/components/layout/CollapseHandle.svelte";
  import StatsView from "$lib/components/stats/StatsView.svelte";
  import SettingsView from "$lib/components/settings/SettingsView.svelte";
  import CalendarTabView from "$lib/components/calendar/CalendarTabView.svelte";
  import ContextMenu from "$lib/components/common/ContextMenu.svelte";
  import UpdateNotification from "$lib/components/common/UpdateNotification.svelte";
  import BreakView from "$lib/components/common/BreakView.svelte";
  import PageModalHost from "$lib/components/modals/PageModalHost.svelte";
  import ProjectListSection from "$lib/components/projects/ProjectListSection.svelte";
  import ResumeView from "$lib/components/resume/ResumeView.svelte";
  import TaskListSection from "$lib/components/tasks/TaskListSection.svelte";
  import ActiveTimerWidget from "$lib/components/timer/ActiveTimerWidget.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import { googleCalendarStore } from "$lib/stores/google-calendar.svelte";

  // ── Multi-window: check if this is the break reminder window ──────────
  const isBreakWindow =
    window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label === "break";
  const isResumeWindow = 
    window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label === "resume";

  type PageModalHostApi = {
    openResetModal: (taskId: number) => void;
    confirmDelete: (type: "project" | "task", id: number) => void;
  };

  let modalHost = $state<PageModalHostApi | null>(null);

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

  function isOverdue(
    deadline: string | null | undefined,
    completed: boolean,
  ): boolean {
    if (!deadline || completed) return false;
    return new Date(deadline) < new Date();
  }

  function getDaysRemaining(updatedAt: number): string {
    const daysPassed = Math.floor((Date.now() / 1000 - updatedAt) / (24 * 60 * 60));
    const remaining = 30 - daysPassed;
    if (remaining <= 0) return "Deleting soon";
    return `${remaining}d left`;
  }

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
      const project = projectStore.projects.find((p) => p.id === id);
      return [
        {
          type: "colorPicker" as const,
          label: "Color",
          currentColor: project?.color ?? "#6366f1",
          onSelect: (color: string) => projectStore.updateColor(id, color),
        },
        {
          label: "Edit Project",
          icon: "✏️",
          onClick: () => uiStore.openProjectModal(id),
        },
        {
          label: "Delete Project",
          icon: "🗑️",
          danger: true,
          onClick: () => modalHost?.confirmDelete("project", id),
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
          onClick: () => modalHost?.confirmDelete("task", id),
        },
      ];
    }
  });
</script>

<svelte:window
  onpointermove={(!isBreakWindow && !isResumeWindow) ? handlePointerMove : undefined}
  onpointerup={(!isBreakWindow && !isResumeWindow) ? handlePointerUp : undefined}
  onpointercancel={(!isBreakWindow && !isResumeWindow) ? cancelDrag : undefined}
  onkeydown={(!isBreakWindow && !isResumeWindow)
    ? (e) => {
        if (e.key === "Escape") cancelDrag();
      }
    : undefined}
  onclick={(!isBreakWindow && !isResumeWindow) ? () => uiStore.closeContextMenu() : undefined}
/>

{#if isBreakWindow}
  <BreakView />
{:else if isResumeWindow}
  <ResumeView />
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
          <ProjectListSection
            {isDragging}
            {draggedId}
            onKeySelect={handleKeySelect}
            onContextMenu={(event, id) => handleContextMenu(event, "project", id)}
            onPointerDown={(event, id, index) =>
              handlePointerDown(event, "project", id, index)}
          />

          {#if projectStore.selectedId !== undefined}
            <TaskListSection
              {isDragging}
              {draggedId}
              {formatDeadline}
              {isOverdue}
              {getDaysRemaining}
              onTaskContextMenu={(event, id) => handleContextMenu(event, "task", id)}
              onTaskPointerDown={(event, id, index) =>
                handlePointerDown(event, "task", id, index)}
              onToggleTimer={handleToggleTimer}
              onStopTimer={handleStopTimer}
              onOpenResetModal={(taskId) => modalHost?.openResetModal(taskId)}
            />
          {/if}
        </div>
      {/if}

      <ActiveTimerWidget onStop={handleStopTimer} />
    {/if}
  </div>

  <ContextMenu items={contextMenuItems} />
  <PageModalHost bind:this={modalHost} />

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
</style>
