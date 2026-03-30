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
  import { createPageInteractions } from "$lib/controllers/page-interactions.svelte";
  import { db } from "$lib/services/db";
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
  const pageInteractions = createPageInteractions({
    onConfirmDelete: (type, id) => modalHost?.confirmDelete(type, id),
  });

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

  function isWindowOrientation(
    value: string | null | undefined,
  ): value is "left" | "right" | "center" {
    return value === "left" || value === "right" || value === "center";
  }

  onMount(async () => {
    if (isBreakWindow || isResumeWindow) {
      uiStore.initTheme();
      return;
    }

    uiStore.initTheme();
    try {
      const savedDockPreference = await db.window.getDockPreference();
      if (isWindowOrientation(savedDockPreference)) {
        uiStore.setWindowOrientation(savedDockPreference);
      } else {
        const orientation = await db.window.getOrientation();
        if (isWindowOrientation(orientation.side)) {
          uiStore.setWindowOrientation(orientation.side);
        }
      }
    } catch (error) {
      console.error("Failed to initialize window orientation:", error);
    }
    timerStore.initBreakReminders();
    await projectStore.loadAll();
    await timerStore.loadActive();
    googleCalendarStore.init();
  });

  $effect(() => {
    if (isBreakWindow || isResumeWindow) {
      return;
    }

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

</script>

<svelte:window
  onpointermove={(!isBreakWindow && !isResumeWindow) ? pageInteractions.handlePointerMove : undefined}
  onpointerup={(!isBreakWindow && !isResumeWindow) ? pageInteractions.handlePointerUp : undefined}
  onpointercancel={(!isBreakWindow && !isResumeWindow) ? pageInteractions.cancelDrag : undefined}
  onkeydown={(!isBreakWindow && !isResumeWindow)
    ? pageInteractions.handleWindowKeydown
    : undefined}
  onclick={(!isBreakWindow && !isResumeWindow) ? pageInteractions.handleWindowClick : undefined}
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
            isDragging={pageInteractions.isDragging}
            draggedId={pageInteractions.draggedId}
            onKeySelect={pageInteractions.handleKeySelect}
            onContextMenu={(event, id) => pageInteractions.handleContextMenu(event, "project", id)}
            onPointerDown={(event, id, index) =>
              pageInteractions.handlePointerDown(event, "project", id, index)}
          />

          {#if projectStore.selectedId !== undefined}
            <TaskListSection
              isDragging={pageInteractions.isDragging}
              draggedId={pageInteractions.draggedId}
              {formatDeadline}
              {isOverdue}
              {getDaysRemaining}
              onTaskContextMenu={(event, id) =>
                pageInteractions.handleContextMenu(event, "task", id)}
              onTaskPointerDown={(event, id, index) =>
                pageInteractions.handlePointerDown(event, "task", id, index)}
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

  <ContextMenu items={pageInteractions.contextMenuItems} />
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
