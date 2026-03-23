import { db, type ActiveTimer, type AutoPauseEvent, AutoPauseReason } from "$lib/services/db";
import { projectStore } from "./projects.svelte";
import { taskStore } from "./tasks.svelte";
import { createBreakReminderController } from "./timer-break-reminders.svelte";

let activeTimer = $state<ActiveTimer | null>(null);
let dailyTotalBeforeActive = $state(0);
let currentElapsed = $state(0);
let intervalId: number | null = null;
let initialTaskTime = 0;
let initialProjectTime = 0;
let currentProjectId = $state<number | null>(null);
let timerChangeCounter = $state(0);
let lastKnownDay = new Date().getDate();
let autoPausedReason = $state<AutoPauseReason | null>(null);

function getStartOfToday(): number {
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  return Math.floor(now.getTime() / 1000);
}

function getContinuousElapsedSeconds(): number {
  if (!activeTimer?.is_running) {
    return 0;
  }

  return activeTimer.elapsed_seconds + (Date.now() / 1000 - activeTimer.started_at);
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

async function refreshDailyTotal(): Promise<void> {
  dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());
}

function stopInterval(): void {
  if (intervalId !== null) {
    clearInterval(intervalId);
    intervalId = null;
  }
}

function startInterval(): void {
  if (intervalId !== null) return;

  lastKnownDay = new Date().getDate();
  intervalId = window.setInterval(() => {
    if (!activeTimer?.is_running) {
      return;
    }

    const currentDay = new Date().getDate();
    if (currentDay !== lastKnownDay) {
      lastKnownDay = currentDay;
      void refreshDailyTotal();
    }

    currentElapsed =
      activeTimer.elapsed_seconds + (Date.now() / 1000 - activeTimer.started_at);

    if (activeTimer.task_id) {
      taskStore.updateTaskTime(
        activeTimer.task_id,
        Math.floor(initialTaskTime + currentElapsed),
      );

      if (currentProjectId) {
        projectStore.updateProjectTime(
          currentProjectId,
          Math.floor(initialProjectTime + currentElapsed),
        );
      }
    }
  }, 1000);
}

const breakReminderController = createBreakReminderController({
  getIsRunning: () => activeTimer?.is_running ?? false,
  getContinuousElapsedSeconds,
});

export interface TimerStore {
  readonly active: ActiveTimer | null;
  readonly elapsed: number;
  readonly dailyTotal: number;
  readonly isRunning: boolean;
  readonly currentProjectId: number | null;
  readonly changeSignal: number;
  readonly autoPausedReason: AutoPauseReason | null;
  readonly isAutoPaused: boolean;
  readonly breakReminderEnabled: boolean;
  readonly breakReminderIntervalMinutes: number;
  readonly breakReminderOpen: boolean;
  readonly breakReminderMessage: string;
  initBreakReminders(): void;
  setBreakReminderEnabled(enabled: boolean): void;
  setBreakReminderInterval(minutes: number): void;
  dismissBreakReminder(): void;
  snoozeBreakReminder(minutes?: number): void;
  loadActive(): Promise<void>;
  start(taskId: number): Promise<ActiveTimer>;
  pause(): Promise<void>;
  resume(): Promise<void>;
  stop(): Promise<null>;
  reset(): Promise<void>;
}

export const timerStore: TimerStore = {
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
      const effectiveStart = Math.max(activeTimer.started_at, startOfToday);
      runningToday = Math.max(0, now - effectiveStart);

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

  get autoPausedReason() {
    return autoPausedReason;
  },

  get isAutoPaused() {
    return autoPausedReason !== null && !activeTimer?.is_running;
  },

  get breakReminderEnabled() {
    return breakReminderController.enabled;
  },

  get breakReminderIntervalMinutes() {
    return breakReminderController.intervalMinutes;
  },

  get breakReminderOpen() {
    return breakReminderController.open;
  },

  get breakReminderMessage() {
    return breakReminderController.message;
  },

  initBreakReminders() {
    breakReminderController.init();
  },

  setBreakReminderEnabled(enabled: boolean) {
    breakReminderController.setEnabled(enabled);
  },

  setBreakReminderInterval(minutes: number) {
    breakReminderController.setInterval(minutes);
  },

  dismissBreakReminder() {
    breakReminderController.dismiss();
  },

  snoozeBreakReminder(minutes?: number) {
    breakReminderController.snooze(minutes);
  },

  async loadActive() {
    breakReminderController.init();

    try {
      const timer = await db.timer.getActive();
      activeTimer = timer;
      await refreshDailyTotal();

      if (!timer) {
        currentProjectId = null;
        breakReminderController.deactivate();
        return;
      }

      currentProjectId = timer.project_id ?? null;
      if (timer.is_running) {
        captureInitialTimes(timer.task_id, currentProjectId);
        startInterval();
        breakReminderController.scheduleAligned();
      } else {
        currentElapsed = timer.elapsed_seconds;
        breakReminderController.deactivate();
      }
    } catch (error) {
      console.error("Failed to load active timer:", error);
    }
  },

  async start(taskId: number) {
    breakReminderController.init();

    try {
      const timer = await db.timer.start(taskId);
      activeTimer = timer;
      currentElapsed = 0;
      currentProjectId = timer.project_id ?? null;
      autoPausedReason = null;

      await refreshDailyTotal();
      captureInitialTimes(taskId, currentProjectId);

      startInterval();
      breakReminderController.closeReminder();
      breakReminderController.scheduleFromCurrentInterval();
      timerChangeCounter++;

      return timer;
    } catch (error) {
      console.error("Failed to start timer:", error);
      throw error;
    }
  },

  async pause() {
    try {
      await db.timer.pause();
      if (activeTimer) {
        activeTimer.is_running = false;
        currentElapsed = getContinuousElapsedSeconds();
      }

      stopInterval();
      breakReminderController.deactivate();
      await refreshDailyTotal();
      currentElapsed = 0;
      timerChangeCounter++;
    } catch (error) {
      console.error("Failed to pause timer:", error);
      throw error;
    }
  },

  async resume() {
    breakReminderController.init();

    try {
      await db.timer.resume();
      if (activeTimer) {
        activeTimer.is_running = true;
        activeTimer.started_at = Math.floor(Date.now() / 1000);
      }

      await refreshDailyTotal();
      currentElapsed = 0;
      autoPausedReason = null;

      startInterval();
      breakReminderController.closeReminder();
      breakReminderController.scheduleFromCurrentInterval();
      timerChangeCounter++;
    } catch (error) {
      console.error("Failed to resume timer:", error);
      throw error;
    }
  },

  async stop() {
    try {
      const projectId = currentProjectId;
      await db.timer.stop();

      activeTimer = null;
      currentElapsed = 0;
      currentProjectId = null;
      stopInterval();
      breakReminderController.deactivate();

      await projectStore.loadAll();
      if (projectId !== null) {
        await taskStore.loadByProject(projectId);
      }

      await refreshDailyTotal();
      timerChangeCounter++;
      return null;
    } catch (error) {
      console.error("Failed to stop timer:", error);
      throw error;
    }
  },

  async reset() {
    try {
      const projectId = currentProjectId;
      await db.timer.reset();

      activeTimer = null;
      currentElapsed = 0;
      currentProjectId = null;
      stopInterval();
      breakReminderController.deactivate();

      await projectStore.loadAll();
      if (projectId !== null) {
        await taskStore.loadByProject(projectId);
      }

      await refreshDailyTotal();
    } catch (error) {
      console.error("Failed to reset timer:", error);
      throw error;
    }
  },
};

if (typeof window !== "undefined") {
  (async () => {
    const { listen } = await import("@tauri-apps/api/event");

    await listen<AutoPauseEvent>("timer:auto-paused", (event) => {
      console.log("Timer auto-paused:", event.payload.reason);
      autoPausedReason = event.payload.reason;

      if (activeTimer) {
        activeTimer.is_running = false;
        currentElapsed = 0;
      }

      stopInterval();
      breakReminderController.deactivate();
      void refreshDailyTotal();
      timerChangeCounter++;
    });

    await listen<{
      action: "take_break" | "dismiss" | "snooze" | "resume";
    }>("break:action", (event) => {
      const { action } = event.payload;

      if (action === "take_break") {
        timerStore.pause().catch((error) =>
          console.error("Failed to pause for break:", error),
        );
      } else if (action === "dismiss") {
        timerStore.dismissBreakReminder();
      } else if (action === "snooze") {
        timerStore.snoozeBreakReminder();
      } else if (action === "resume") {
        timerStore.resume().catch((error) =>
          console.error("Failed to resume after break:", error),
        );
      }
    });
  })();
}
