import { db, type ActiveTimer, AutoPauseReason } from "$lib/services/db";
import { projectStore } from "./projects.svelte";
import { taskStore } from "./tasks.svelte";
import { createBreakReminderController } from "./timer-break-reminders.svelte";
import { registerTimerEventHandlers } from "./timer-events";
import { createTimerRuntimeController } from "./timer-runtime.svelte";

let activeTimer = $state<ActiveTimer | null>(null);
let dailyTotalBeforeActive = $state(0);
let currentProjectId = $state<number | null>(null);
let timerChangeCounter = $state(0);
let autoPausedReason = $state<AutoPauseReason | null>(null);

async function refreshDailyTotal(): Promise<void> {
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(
    Math.floor(now.getTime() / 1000),
  );
}

const timerRuntime = createTimerRuntimeController({
  getActiveTimer: () => activeTimer,
  getCurrentProjectId: () => currentProjectId,
  getDailyTotalBeforeActive: () => dailyTotalBeforeActive,
  refreshDailyTotal,
});

const breakReminderController = createBreakReminderController({
  getIsRunning: () => activeTimer?.is_running ?? false,
  getContinuousElapsedSeconds: () => timerRuntime.getContinuousElapsedSeconds(),
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
    return timerRuntime.elapsed;
  },

  get dailyTotal() {
    return timerRuntime.getDailyTotal();
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
        timerRuntime.resetElapsed();
        return;
      }

      currentProjectId = timer.project_id ?? null;
      if (timer.is_running) {
        timerRuntime.captureInitialTimes(timer.task_id, currentProjectId);
        timerRuntime.startInterval();
        breakReminderController.scheduleAligned();
      } else {
        timerRuntime.setElapsed(timer.elapsed_seconds);
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
      timerRuntime.resetElapsed();
      currentProjectId = timer.project_id ?? null;
      autoPausedReason = null;

      await refreshDailyTotal();
      timerRuntime.captureInitialTimes(taskId, currentProjectId);

      timerRuntime.startInterval();
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
      const pausedElapsed = timerRuntime.getDisplayElapsedSeconds();
      if (activeTimer) {
        activeTimer.elapsed_seconds = 0;
        activeTimer.is_running = false;
      }
      timerRuntime.setElapsed(pausedElapsed);

      timerRuntime.stopInterval();
      breakReminderController.deactivate();
      await refreshDailyTotal();
      timerChangeCounter++;
    } catch (error) {
      console.error("Failed to pause timer:", error);
      throw error;
    }
  },

  async resume() {
    breakReminderController.init();

    try {
      const resumedElapsed = timerRuntime.elapsed;
      await db.timer.resume();
      if (activeTimer) {
        activeTimer.elapsed_seconds = 0;
        activeTimer.is_running = true;
        activeTimer.started_at = Math.floor(Date.now() / 1000);
      }

      await refreshDailyTotal();
      timerRuntime.setElapsed(resumedElapsed);
      autoPausedReason = null;

      timerRuntime.startInterval();
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
      timerRuntime.resetElapsed();
      currentProjectId = null;
      timerRuntime.stopInterval();
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
      timerRuntime.resetElapsed();
      currentProjectId = null;
      timerRuntime.stopInterval();
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

registerTimerEventHandlers({
  onAutoPaused: (reason) => {
    const pausedElapsed = timerRuntime.getDisplayElapsedSeconds();
    autoPausedReason = reason;

    if (activeTimer) {
      activeTimer.elapsed_seconds = 0;
      activeTimer.is_running = false;
    }

    timerRuntime.setElapsed(pausedElapsed);
    timerRuntime.stopInterval();
    breakReminderController.deactivate();
    void refreshDailyTotal();
    timerChangeCounter++;
  },
  onTakeBreak: () => timerStore.pause(),
  onDismiss: () => timerStore.dismissBreakReminder(),
  onSnooze: () => timerStore.snoozeBreakReminder(),
  onResume: () => timerStore.resume(),
});
