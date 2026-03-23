import { invoke } from "@tauri-apps/api/core";

const BREAK_REMINDER_ENABLED_KEY = "breakReminderEnabled";
const BREAK_REMINDER_INTERVAL_KEY = "breakReminderIntervalMinutes";
const BREAK_REMINDER_MIN_ACTIVE_MINUTES = 15;
const BREAK_REMINDER_MAX_MINUTES = 60;
const BREAK_REMINDER_DEFAULT_MINUTES = 30;
const BREAK_REMINDER_DEFAULT_MESSAGE =
  "Quick break time. Stand up, stretch, and reset.";
const BREAK_REMINDER_MESSAGES = [
  BREAK_REMINDER_DEFAULT_MESSAGE,
  "You have been focused for a while. Grab water and breathe for a minute.",
  "Eyes off the screen for a bit. Your next session will be sharper.",
  "Small pause, big gain. Loosen your shoulders and take a short walk.",
  "Momentum is great, but recovery matters too. Take a quick break.",
  "Pause now so you can keep a steady pace later.",
];

interface CreateBreakReminderControllerOptions {
  getIsRunning: () => boolean;
  getContinuousElapsedSeconds: () => number;
}

export function createBreakReminderController({
  getIsRunning,
  getContinuousElapsedSeconds,
}: CreateBreakReminderControllerOptions) {
  let enabled = $state(false);
  let intervalMinutes = $state(BREAK_REMINDER_DEFAULT_MINUTES);
  let reminderOpen = $state(false);
  let reminderMessage = $state(BREAK_REMINDER_DEFAULT_MESSAGE);
  let timeoutId: number | null = null;
  let initialized = false;
  let lastMessageIndex = -1;

  function clampMinutes(minutes: number): number {
    if (minutes === 0) return 0;
    return Math.min(
      BREAK_REMINDER_MAX_MINUTES,
      Math.max(BREAK_REMINDER_MIN_ACTIVE_MINUTES, Math.round(minutes)),
    );
  }

  function clearScheduledReminder(): void {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
  }

  function persistSettings(): void {
    if (typeof window === "undefined") return;

    localStorage.setItem(BREAK_REMINDER_ENABLED_KEY, String(enabled));
    localStorage.setItem(BREAK_REMINDER_INTERVAL_KEY, String(intervalMinutes));
  }

  function pickMessage(): string {
    if (BREAK_REMINDER_MESSAGES.length === 1) {
      return BREAK_REMINDER_MESSAGES[0];
    }

    let nextIndex = Math.floor(Math.random() * BREAK_REMINDER_MESSAGES.length);
    while (nextIndex === lastMessageIndex) {
      nextIndex = Math.floor(Math.random() * BREAK_REMINDER_MESSAGES.length);
    }

    lastMessageIndex = nextIndex;
    return BREAK_REMINDER_MESSAGES[nextIndex];
  }

  function schedule(delayMs: number): void {
    clearScheduledReminder();

    if (!getIsRunning() || !enabled) {
      return;
    }

    const safeDelayMs = Math.max(1000, Math.floor(delayMs));
    timeoutId = window.setTimeout(() => {
      timeoutId = null;

      if (!getIsRunning() || !enabled) {
        return;
      }

      const nextMessage = pickMessage();
      reminderMessage = nextMessage;
      const currentTheme = localStorage.getItem("theme") ?? "light";
      invoke("open_break_window", {
        message: nextMessage,
        theme: currentTheme,
      }).catch((error) => {
        console.error("Failed to open break window:", error);
        reminderOpen = true;
      });
    }, safeDelayMs);
  }

  function scheduleFromNow(minutes: number): void {
    const clampedMinutes = clampMinutes(minutes);
    if (clampedMinutes === 0) {
      deactivate();
      return;
    }

    schedule(clampedMinutes * 60 * 1000);
  }

  function scheduleFromCurrentInterval(): void {
    scheduleFromNow(intervalMinutes);
  }

  function scheduleAligned(): void {
    if (!getIsRunning() || !enabled) {
      clearScheduledReminder();
      return;
    }

    const intervalMs = intervalMinutes * 60 * 1000;
    const elapsedMs = getContinuousElapsedSeconds() * 1000;
    let delayMs = intervalMs - (elapsedMs % intervalMs);
    if (delayMs < 1000) {
      delayMs = intervalMs;
    }

    schedule(delayMs);
  }

  function closeReminder(): void {
    reminderOpen = false;
  }

  function deactivate(): void {
    reminderOpen = false;
    clearScheduledReminder();
  }

  function init(): void {
    if (initialized || typeof window === "undefined") return;
    initialized = true;

    const savedEnabled = localStorage.getItem(BREAK_REMINDER_ENABLED_KEY);
    const savedInterval = localStorage.getItem(BREAK_REMINDER_INTERVAL_KEY);

    enabled = savedEnabled !== "false";

    if (savedInterval !== null) {
      const parsed = Number(savedInterval);
      if (Number.isFinite(parsed) && parsed >= 0) {
        intervalMinutes = clampMinutes(parsed);
        if (intervalMinutes === 0) {
          enabled = false;
        }
      }
    } else {
      intervalMinutes = BREAK_REMINDER_DEFAULT_MINUTES;
    }
  }

  function setEnabled(nextEnabled: boolean): void {
    enabled = nextEnabled;
    reminderOpen = false;

    if (nextEnabled && getIsRunning()) {
      scheduleFromCurrentInterval();
    } else {
      clearScheduledReminder();
    }

    persistSettings();
  }

  function setInterval(minutes: number): void {
    intervalMinutes = clampMinutes(minutes);

    if (intervalMinutes === 0) {
      enabled = false;
      deactivate();
    } else if (enabled && getIsRunning() && !reminderOpen) {
      scheduleFromCurrentInterval();
    }

    persistSettings();
  }

  function dismiss(): void {
    reminderOpen = false;

    if (enabled && getIsRunning()) {
      scheduleFromCurrentInterval();
    } else {
      clearScheduledReminder();
    }
  }

  function snooze(minutes: number = 10): void {
    reminderOpen = false;

    if (enabled && getIsRunning()) {
      scheduleFromNow(minutes);
    } else {
      clearScheduledReminder();
    }
  }

  return {
    get enabled() {
      return enabled;
    },
    get intervalMinutes() {
      return intervalMinutes;
    },
    get open() {
      return reminderOpen;
    },
    get message() {
      return reminderMessage;
    },
    init,
    setEnabled,
    setInterval,
    dismiss,
    snooze,
    closeReminder,
    deactivate,
    scheduleAligned,
    scheduleFromCurrentInterval,
  };
}
