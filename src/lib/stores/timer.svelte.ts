import { db, type ActiveTimer } from "$lib/services/db";

let activeTimer = $state<ActiveTimer | null>(null);
let currentElapsed = $state(0);
let intervalId: number | null = null;

export const timerStore = {
  get active() {
    return activeTimer;
  },

  get elapsed() {
    return currentElapsed;
  },

  get isRunning() {
    return activeTimer?.is_running ?? false;
  },

  async loadActive() {
    try {
      const timer = await db.timer.getActive();
      activeTimer = timer;
      if (timer && timer.is_running) {
        this.startInterval();
      } else if (timer) {
        currentElapsed = timer.elapsed_seconds;
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
      this.startInterval();
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
      this.startInterval();
    } catch (e) {
      console.error("Failed to resume timer:", e);
      throw e;
    }
  },

  async stop() {
    try {
      const entry = await db.timer.stop();
      activeTimer = null;
      currentElapsed = 0;
      this.stopInterval();
      return entry;
    } catch (e) {
      console.error("Failed to stop timer:", e);
      throw e;
    }
  },

  startInterval() {
    if (intervalId !== null) return;

    intervalId = window.setInterval(() => {
      if (activeTimer?.is_running) {
        currentElapsed = activeTimer.elapsed_seconds + (Date.now() / 1000 - activeTimer.started_at);
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
