import { db, type ActiveWindowTracking } from "$lib/services/db";

let enabled = $state(false);
let paused = $state(false);
let active = $state<ActiveWindowTracking | null>(null);
let todayTotalBeforeActive = $state(0);
let changeCounter = $state(0);
let refreshIntervalId: number | null = null;
const WINDOW_TRACKING_REFRESH_MS = 15_000;

function activeElapsedSeconds(): number {
  if (!active) return 0;
  return Math.max(0, Date.now() / 1000 - active.app_started_at);
}

function continuousWorkSeconds(): number {
  if (!enabled || paused || !active) return 0;
  return Math.max(0, Date.now() / 1000 - active.work_started_at);
}

function stopRefresh(): void {
  if (refreshIntervalId !== null) {
    clearInterval(refreshIntervalId);
    refreshIntervalId = null;
  }
}

function startRefresh(): void {
  if (typeof window === "undefined" || refreshIntervalId !== null) return;
  refreshIntervalId = window.setInterval(() => {
    void windowTrackingStore.refresh();
  }, WINDOW_TRACKING_REFRESH_MS);
}

export const windowTrackingStore = {
  get enabled() {
    return enabled;
  },

  get paused() {
    return paused;
  },

  get active() {
    return active;
  },

  get isWorkActive() {
    return enabled && !paused && active !== null;
  },

  get dailyTotal() {
    return todayTotalBeforeActive + activeElapsedSeconds();
  },

  get activeElapsed() {
    return activeElapsedSeconds();
  },

  get continuousWorkElapsed() {
    return continuousWorkSeconds();
  },

  get changeSignal() {
    return changeCounter;
  },

  async init() {
    await this.refresh();
    if (enabled) {
      startRefresh();
    }
  },

  async refresh() {
    try {
      const state = await db.windowTracking.getState();
      enabled = state.enabled;
      paused = state.paused;
      active = state.active ?? null;
      const activeSeconds = active
        ? Math.max(0, Date.now() / 1000 - active.app_started_at)
        : 0;
      todayTotalBeforeActive = Math.max(
        0,
        state.today_total_seconds - activeSeconds,
      );

      if (enabled && !paused) {
        startRefresh();
      } else {
        stopRefresh();
      }

      changeCounter++;
    } catch (error) {
      console.error("Failed to refresh window tracking:", error);
    }
  },

  async setEnabled(nextEnabled: boolean) {
    try {
      const settings = await db.windowTracking.setEnabled(nextEnabled);
      enabled = settings.enabled;
      if (!enabled) {
        paused = false;
        active = null;
        todayTotalBeforeActive = 0;
        stopRefresh();
      } else {
        startRefresh();
      }

      await this.refresh();
      changeCounter++;
    } catch (error) {
      console.error("Failed to update window tracking:", error);
      throw error;
    }
  },

  async setPaused(nextPaused: boolean) {
    try {
      const state = await db.windowTracking.setPaused(nextPaused);
      enabled = state.enabled;
      paused = state.paused;
      active = state.active ?? null;
      const activeSeconds = active
        ? Math.max(0, Date.now() / 1000 - active.app_started_at)
        : 0;
      todayTotalBeforeActive = Math.max(
        0,
        state.today_total_seconds - activeSeconds,
      );

      if (enabled && !paused) {
        startRefresh();
      } else {
        stopRefresh();
      }

      changeCounter++;
    } catch (error) {
      console.error("Failed to pause window tracking:", error);
      throw error;
    }
  },

  async clearActivity() {
    await db.windowTracking.clearActivity();
    active = null;
    todayTotalBeforeActive = 0;
    changeCounter++;
  },
};
