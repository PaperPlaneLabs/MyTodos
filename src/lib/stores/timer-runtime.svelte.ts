import type { ActiveTimer } from "$lib/services/db";
import {
  calculateContinuousElapsedSeconds,
  calculateDisplayElapsedSeconds,
} from "./timer-runtime-utils";
import { projectStore } from "./projects.svelte";
import { taskStore } from "./tasks.svelte";

interface CreateTimerRuntimeControllerOptions {
  getActiveTimer: () => ActiveTimer | null;
  getCurrentProjectId: () => number | null;
  getDailyTotalBeforeActive: () => number;
  refreshDailyTotal: () => Promise<void>;
}

export function createTimerRuntimeController({
  getActiveTimer,
  getCurrentProjectId,
  getDailyTotalBeforeActive,
  refreshDailyTotal,
}: CreateTimerRuntimeControllerOptions) {
  let currentElapsed = $state(0);
  let elapsedOffset = 0;
  let intervalId: number | null = null;
  let initialTaskTime = 0;
  let initialProjectTime = 0;
  let lastKnownDay = new Date().getDate();

  function getStartOfToday(): number {
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    return Math.floor(now.getTime() / 1000);
  }

  function getContinuousElapsedSeconds(): number {
    return calculateContinuousElapsedSeconds(getActiveTimer());
  }

  function getDisplayElapsedSeconds(): number {
    return calculateDisplayElapsedSeconds(getActiveTimer(), elapsedOffset);
  }

  function captureInitialTimes(taskId: number, projectId: number | null): void {
    initialTaskTime = 0;
    initialProjectTime = 0;

    const task = taskStore.tasks.find((item) => item.id === taskId);
    if (!task) {
      return;
    }

    initialTaskTime = task.total_time_seconds;
    if (projectId) {
      const project = projectStore.projects.find((item) => item.id === projectId);
      initialProjectTime = project?.total_time_seconds ?? 0;
    }
  }

  function setElapsed(seconds: number): void {
    currentElapsed = seconds;
    elapsedOffset = seconds;
  }

  function resetElapsed(): void {
    currentElapsed = 0;
    elapsedOffset = 0;
  }

  function startInterval(): void {
    if (intervalId !== null) return;

    lastKnownDay = new Date().getDate();
    intervalId = window.setInterval(() => {
      const activeTimer = getActiveTimer();
      if (!activeTimer?.is_running) {
        return;
      }

      const currentDay = new Date().getDate();
      if (currentDay !== lastKnownDay) {
        lastKnownDay = currentDay;
        void refreshDailyTotal();
      }

      currentElapsed = getDisplayElapsedSeconds();

      if (activeTimer.task_id) {
        taskStore.updateTaskTime(
          activeTimer.task_id,
          Math.floor(initialTaskTime + currentElapsed),
        );

        const currentProjectId = getCurrentProjectId();
        if (currentProjectId) {
          projectStore.updateProjectTime(
            currentProjectId,
            Math.floor(initialProjectTime + currentElapsed),
          );
        }
      }
    }, 1000);
  }

  function stopInterval(): void {
    if (intervalId !== null) {
      clearInterval(intervalId);
      intervalId = null;
    }
  }

  function getDailyTotal(): number {
    const activeTimer = getActiveTimer();
    let runningToday = 0;

    if (activeTimer?.is_running) {
      const now = Date.now() / 1000;
      const startOfToday = getStartOfToday();
      const effectiveStart = Math.max(activeTimer.started_at, startOfToday);
      runningToday = Math.max(0, now - effectiveStart);

      if (activeTimer.started_at >= startOfToday) {
        runningToday += activeTimer.elapsed_seconds;
      }
    }

    return getDailyTotalBeforeActive() + runningToday;
  }

  return {
    get elapsed() {
      return currentElapsed;
    },
    getContinuousElapsedSeconds,
    getDisplayElapsedSeconds,
    captureInitialTimes,
    setElapsed,
    resetElapsed,
    startInterval,
    stopInterval,
    getDailyTotal,
  };
}
