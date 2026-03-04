import { db, type ActiveTimer, type AutoPauseEvent, AutoPauseReason } from "$lib/services/db";
import { taskStore } from "./tasks.svelte";
import { projectStore } from "./projects.svelte";
import { invoke } from "@tauri-apps/api/core";

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
let breakReminderEnabled = $state(false); // Disabled for now - can be re-enabled later
let breakReminderIntervalMinutes = $state(30);
let breakReminderOpen = $state(false);
let breakReminderMessage = $state("Quick break time. Stand up, stretch, and reset.");
let breakReminderTimeoutId: number | null = null;
let breakReminderInitialized = false;
let lastBreakMessageIndex = -1;

const BREAK_REMINDER_ENABLED_KEY = "breakReminderEnabled";
const BREAK_REMINDER_INTERVAL_KEY = "breakReminderIntervalMinutes";
const BREAK_REMINDER_MIN_MINUTES = 10;
const BREAK_REMINDER_MAX_MINUTES = 60;
const BREAK_REMINDER_DEFAULT_MINUTES = 30;
const BREAK_REMINDER_SNOOZE_MINUTES = 10;
const BREAK_REMINDER_MESSAGES = [
  "Quick break time. Stand up, stretch, and reset.",
  "You have been focused for a while. Grab water and breathe for a minute.",
  "Eyes off the screen for a bit. Your next session will be sharper.",
  "Small pause, big gain. Loosen your shoulders and take a short walk.",
  "Momentum is great, but recovery matters too. Take a quick break.",
  "Pause now so you can keep a steady pace later.",
];

function getStartOfToday(): number {
  const now = new Date();
  now.setHours(0, 0, 0, 0);
  return Math.floor(now.getTime() / 1000);
}

function clampBreakReminderMinutes(minutes: number): number {
  return Math.min(
    BREAK_REMINDER_MAX_MINUTES,
    Math.max(BREAK_REMINDER_MIN_MINUTES, Math.round(minutes)),
  );
}

function clearBreakReminderTimeout(): void {
  if (breakReminderTimeoutId !== null) {
    clearTimeout(breakReminderTimeoutId);
    breakReminderTimeoutId = null;
  }
}

function persistBreakReminderSettings(): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(BREAK_REMINDER_ENABLED_KEY, String(breakReminderEnabled));
  localStorage.setItem(
    BREAK_REMINDER_INTERVAL_KEY,
    String(breakReminderIntervalMinutes),
  );
}

function pickBreakReminderMessage(): string {
  if (BREAK_REMINDER_MESSAGES.length === 1) {
    return BREAK_REMINDER_MESSAGES[0];
  }

  let nextIndex = Math.floor(Math.random() * BREAK_REMINDER_MESSAGES.length);
  while (nextIndex === lastBreakMessageIndex) {
    nextIndex = Math.floor(Math.random() * BREAK_REMINDER_MESSAGES.length);
  }

  lastBreakMessageIndex = nextIndex;
  return BREAK_REMINDER_MESSAGES[nextIndex];
}

function getContinuousElapsedSeconds(): number {
  if (!activeTimer?.is_running) {
    return 0;
  }
  return activeTimer.elapsed_seconds + (Date.now() / 1000 - activeTimer.started_at);
}

function scheduleBreakReminder(delayMs: number): void {
  clearBreakReminderTimeout();

  if (!activeTimer?.is_running || !breakReminderEnabled) {
    return;
  }

  const safeDelayMs = Math.max(1000, Math.floor(delayMs));
  breakReminderTimeoutId = window.setTimeout(() => {
    breakReminderTimeoutId = null;

    if (!activeTimer?.is_running || !breakReminderEnabled) {
      return;
    }

    const msg = pickBreakReminderMessage();
    breakReminderMessage = msg;
    const currentTheme = localStorage.getItem("theme") ?? "light";
    invoke("open_break_window", { message: msg, theme: currentTheme }).catch((e) => {
      // Fallback: show in-app modal if window creation fails
      console.error("Failed to open break window:", e);
      breakReminderOpen = true;
    });
  }, safeDelayMs);
}

function scheduleAlignedBreakReminder(): void {
  if (!activeTimer?.is_running || !breakReminderEnabled) {
    clearBreakReminderTimeout();
    return;
  }

  const intervalMs = breakReminderIntervalMinutes * 60 * 1000;
  const elapsedMs = getContinuousElapsedSeconds() * 1000;
  let delayMs = intervalMs - (elapsedMs % intervalMs);
  if (delayMs < 1000) {
    delayMs = intervalMs;
  }

  scheduleBreakReminder(delayMs);
}

function scheduleBreakReminderFromNow(minutes: number): void {
  scheduleBreakReminder(clampBreakReminderMinutes(minutes) * 60 * 1000);
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

  get autoPausedReason() {
    return autoPausedReason;
  },

  get isAutoPaused() {
    return autoPausedReason !== null && !activeTimer?.is_running;
  },

  get breakReminderEnabled() {
    return breakReminderEnabled;
  },

  get breakReminderIntervalMinutes() {
    return breakReminderIntervalMinutes;
  },

  get breakReminderOpen() {
    return breakReminderOpen;
  },

  get breakReminderMessage() {
    return breakReminderMessage;
  },

  initBreakReminders() {
    if (breakReminderInitialized || typeof window === "undefined") return;
    breakReminderInitialized = true;

    const savedEnabled = localStorage.getItem(BREAK_REMINDER_ENABLED_KEY);
    const savedInterval = localStorage.getItem(BREAK_REMINDER_INTERVAL_KEY);

    breakReminderEnabled = savedEnabled !== "false";

    if (savedInterval !== null) {
      const parsed = Number(savedInterval);
      if (Number.isFinite(parsed)) {
        breakReminderIntervalMinutes = clampBreakReminderMinutes(parsed);
      }
    } else {
      breakReminderIntervalMinutes = BREAK_REMINDER_DEFAULT_MINUTES;
    }
  },

  setBreakReminderEnabled(enabled: boolean) {
    breakReminderEnabled = enabled;
    breakReminderOpen = false;
    if (enabled && activeTimer?.is_running) {
      scheduleBreakReminderFromNow(breakReminderIntervalMinutes);
    } else {
      clearBreakReminderTimeout();
    }
    persistBreakReminderSettings();
  },

  setBreakReminderInterval(minutes: number) {
    breakReminderIntervalMinutes = clampBreakReminderMinutes(minutes);
    if (breakReminderEnabled && activeTimer?.is_running && !breakReminderOpen) {
      scheduleBreakReminderFromNow(breakReminderIntervalMinutes);
    }
    persistBreakReminderSettings();
  },

  dismissBreakReminder() {
    breakReminderOpen = false;
    if (breakReminderEnabled && activeTimer?.is_running) {
      scheduleBreakReminderFromNow(breakReminderIntervalMinutes);
    } else {
      clearBreakReminderTimeout();
    }
  },

  snoozeBreakReminder(minutes: number = BREAK_REMINDER_SNOOZE_MINUTES) {
    breakReminderOpen = false;
    if (breakReminderEnabled && activeTimer?.is_running) {
      scheduleBreakReminderFromNow(minutes);
    } else {
      clearBreakReminderTimeout();
    }
  },

  async loadActive() {
    this.initBreakReminders();
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
          scheduleAlignedBreakReminder();
        } else {
          currentElapsed = timer.elapsed_seconds;
          breakReminderOpen = false;
          clearBreakReminderTimeout();
        }
      } else {
        breakReminderOpen = false;
        clearBreakReminderTimeout();
      }
    } catch (e) {
      console.error("Failed to load active timer:", e);
    }
  },

  async start(taskId: number) {
    this.initBreakReminders();
    try {
      const timer = await db.timer.start(taskId);
      activeTimer = timer;
      currentElapsed = 0;
      breakReminderOpen = false;

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
      scheduleBreakReminderFromNow(breakReminderIntervalMinutes);
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
      breakReminderOpen = false;
      clearBreakReminderTimeout();

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
    this.initBreakReminders();
    try {
      await db.timer.resume();
      if (activeTimer) {
        activeTimer.is_running = true;
        activeTimer.started_at = Math.floor(Date.now() / 1000);
      }
      // Refresh daily total before resuming
      dailyTotalBeforeActive = await db.timeEntries.getDailyTotalTime(getStartOfToday());
      currentElapsed = 0;
      autoPausedReason = null; // Clear auto-pause state
      this.startInterval();
      breakReminderOpen = false;
      scheduleBreakReminderFromNow(breakReminderIntervalMinutes);
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
      breakReminderOpen = false;
      clearBreakReminderTimeout();

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
      breakReminderOpen = false;
      clearBreakReminderTimeout();

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

// Listen for auto-pause events from the backend and break window actions
if (typeof window !== 'undefined') {
  (async () => {
    const { listen } = await import("@tauri-apps/api/event");

    await listen<AutoPauseEvent>("timer:auto-paused", (event) => {
      console.log("Timer auto-paused:", event.payload.reason);
      autoPausedReason = event.payload.reason;

      if (activeTimer) {
        activeTimer.is_running = false;
        currentElapsed = 0;
      }

      timerStore.stopInterval();
      breakReminderOpen = false;
      clearBreakReminderTimeout();
      timerStore.refreshDailyTotal();
      timerChangeCounter++;
    });

    await listen<{ action: "take_break" | "dismiss" | "snooze" | "resume" }>("break:action", (event) => {
      const { action } = event.payload;
      if (action === "take_break") {
        timerStore.pause().catch((e) => console.error("Failed to pause for break:", e));
      } else if (action === "dismiss") {
        timerStore.dismissBreakReminder();
      } else if (action === "snooze") {
        timerStore.snoozeBreakReminder(BREAK_REMINDER_SNOOZE_MINUTES);
      } else if (action === "resume") {
        timerStore.resume().catch((e) => console.error("Failed to resume after break:", e));
      }
    });
  })();
}
