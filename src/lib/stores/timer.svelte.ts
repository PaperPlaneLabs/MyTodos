import { db, type ActiveTimer } from "$lib/services/db";
import { taskStore } from "./tasks.svelte";
import { projectStore } from "./projects.svelte";

let activeTimer = $state<ActiveTimer | null>(null);
let dailyTotalBeforeActive = $state(0);
let currentElapsed = $state(0);
let intervalId: number | null = null;
let initialTaskTime = 0;
let initialProjectTime = 0;
let currentProjectId = $state<number | null>(null);
let timerChangeCounter = $state(0);
let lastKnownDay = new Date().getDate();

function getStartOfToday(): number {
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  return Math.floor(now.getTime() / 1000);
}

export const timerStore = {
  get active() {
    return activeTimer;
  },

  get elapsed() {
    return currentElapsed;
  },

  get dailyTotal() {
    let runningToday = 0;
    if (activeTimer?.is_running) {
      const now = Date.now() / 1000;
      const startOfToday = getStartOfToday();

      // CRITICAL: Only count time from today onward
      // If timer started yesterday, only count from midnight today
      const effectiveStart = Math.max(activeTimer.started_at, startOfToday);
      runningToday = Math.max(0, now - effectiveStart);

      // Don't add elapsed_seconds if timer started before today
      // (with new pause behavior, elapsed_seconds should be 0 anyway)
      if (activeTimer.started_at >= startOfToday) {
        runningToday += activeTimer.elapsed_seconds;
      }
    }
    return dailyTotalBeforeActive + runningToday;
  },

  get isRunning() {
    return activeTimer?.is_running ?? false;
  },

  get currentProjectId() {
    return currentProjectId;
  },

  get changeSignal() {
    return timerChangeCounter;
  },

  async loadActive() {
    try {
      const timer = await db.timer.getActive();
      activeTimer = timer;

      // If timer exists and started before today, refresh daily total to ensure it's for today only
      const startOfToday = getStartOfToday();
      if (timer && timer.started_at < startOfToday) {
        // Timer started yesterday or earlier - refresh daily total
        dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(startOfToday);
      } else {
        // Timer started today or no timer - load daily total normally
        dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(startOfToday);
      }

      if (timer) {
        // Use project_id directly from the timer - it's now persisted in the database
        currentProjectId = timer.project_id ?? null;

        if (timer.is_running) {
          // Capture initial times for the UI updates
          const task = taskStore.tasks.find(t => t.id === timer.task_id);
          if (task) {
            initialTaskTime = task.total_time_seconds;
            if (currentProjectId) {
              const project = projectStore.projects.find(p => p.id === currentProjectId);
              initialProjectTime = project?.total_time_seconds ?? 0;
            }
          }
          this.startInterval();
        } else {
          currentElapsed = timer.elapsed_seconds;
        }
      }
    } catch (e) {
      console.error("Failed to load active timer:", e);
    }
  },

  async start(taskId: number) {
    try {
      const timer = await db.timer.start(taskId);
      activeTimer = timer;
      currentElapsed = 0;

      // Refresh daily total before starting
      dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());

      // Use project_id directly from the timer response
      currentProjectId = timer.project_id ?? null;

      // Capture initial times for the UI updates
      const task = taskStore.tasks.find(t => t.id === taskId);
      if (task) {
        initialTaskTime = task.total_time_seconds;
        if (currentProjectId) {
          const project = projectStore.projects.find(p => p.id === currentProjectId);
          initialProjectTime = project?.total_time_seconds ?? 0;
        }
      }

      this.startInterval();
      timerChangeCounter++;
      return timer;
    } catch (e) {
      console.error("Failed to start timer:", e);
      throw e;
    }
  },

  async pause() {
    try {
      await db.timer.pause();
      if (activeTimer) {
        activeTimer.is_running = false;
        currentElapsed = this.elapsed;
      }
      this.stopInterval();

      // Refresh daily total after pause (entry might have been created)
      dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());
      currentElapsed = 0; // Reset active elapsed as it's now in the database total
      timerChangeCounter++;
    } catch (e) {
      console.error("Failed to pause timer:", e);
      throw e;
    }
  },

  async resume() {
    try {
      await db.timer.resume();
      if (activeTimer) {
        activeTimer.is_running = true;
        activeTimer.started_at = Math.floor(Date.now() / 1000);
      }
      // Refresh daily total before resuming
      dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());
      currentElapsed = 0;
      this.startInterval();
      timerChangeCounter++;
    } catch (e) {
      console.error("Failed to resume timer:", e);
      throw e;
    }
  },

  async stop() {
    try {
      const projectId = currentProjectId;
      await db.timer.stop();
      activeTimer = null;
      currentElapsed = 0;
      this.stopInterval();

      // Reload stores to get authoritative values from database
      await projectStore.loadAll();
      if (projectId !== null) {
        await taskStore.loadByProject(projectId);
      }

      // Refresh daily total
      dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());
      timerChangeCounter++;

      return null; // Return value changed in stopped entry logic if needed but caller doesn't seem to use it much
    } catch (e) {
      console.error("Failed to stop timer:", e);
      throw e;
    }
  },

  async refreshDailyTotal() {
    dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());
  },

  async reset() {
    try {
      const projectId = currentProjectId;
      await db.timer.reset();
      activeTimer = null;
      currentElapsed = 0;
      this.stopInterval();

      // Reload stores to ensure consistency
      await projectStore.loadAll();
      if (projectId !== null) {
        await taskStore.loadByProject(projectId);
      }

      // Refresh daily total
      await this.refreshDailyTotal();
    } catch (e) {
      console.error("Failed to reset timer:", e);
      throw e;
    }
  },

  startInterval() {
    if (intervalId !== null) return;

    // Initialize lastKnownDay to current day
    lastKnownDay = new Date().getDate();

    intervalId = window.setInterval(() => {
      if (activeTimer?.is_running) {
        // Check if day changed (midnight crossed)
        const currentDay = new Date().getDate();
        if (currentDay !== lastKnownDay) {
          lastKnownDay = currentDay;
          this.refreshDailyTotal(); // Reset to new day's total
        }

        currentElapsed = activeTimer.elapsed_seconds + (Date.now() / 1000 - activeTimer.started_at);

        // Update task and project stores with current elapsed time
        if (activeTimer.task_id) {
          taskStore.updateTaskTime(activeTimer.task_id, Math.floor(initialTaskTime + currentElapsed));

          if (currentProjectId) {
            projectStore.updateProjectTime(currentProjectId, Math.floor(initialProjectTime + currentElapsed));
          }
        }
      }
    }, 1000);
  },

  stopInterval() {
    if (intervalId !== null) {
      clearInterval(intervalId);
      intervalId = null;
    }
  },
};
