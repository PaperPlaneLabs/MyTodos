import { beforeEach, describe, expect, it, vi } from "vitest";

import type { ActiveTimer } from "$lib/services/db";
import {
  calculateContinuousElapsedSeconds,
  calculateDisplayElapsedSeconds,
  calculateTimerRemainingSeconds,
} from "./timer-runtime-utils";

describe("timer-runtime-utils", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-23T09:00:00.000Z"));
  });

  it("keeps the paused elapsed value until the timer resumes", () => {
    const pausedTimer: ActiveTimer = {
      task_id: 1,
      started_at: Math.floor(Date.now() / 1000) - 125,
      elapsed_seconds: 0,
      is_running: false,
    };

    expect(calculateDisplayElapsedSeconds(pausedTimer, 125)).toBe(125);
    expect(calculateContinuousElapsedSeconds(pausedTimer)).toBe(0);
  });

  it("adds resumed runtime on top of the preserved paused elapsed value", () => {
    const resumedTimer: ActiveTimer = {
      task_id: 1,
      started_at: Math.floor(Date.now() / 1000) - 3,
      elapsed_seconds: 0,
      is_running: true,
    };

    expect(Math.floor(calculateDisplayElapsedSeconds(resumedTimer, 125))).toBe(128);
    expect(Math.floor(calculateContinuousElapsedSeconds(resumedTimer))).toBe(3);
  });

  it("counts down a running bounded task timer from its persisted expiry", () => {
    const now = Math.floor(Date.now() / 1000);
    const timer: ActiveTimer = {
      task_id: 2,
      started_at: now - 300,
      elapsed_seconds: 0,
      is_running: true,
      timer_limit_seconds: 1500,
      timer_remaining_seconds: 1500,
      timer_expires_at: now + 1200,
    };

    expect(calculateTimerRemainingSeconds(timer)).toBe(1200);
    expect(calculateDisplayElapsedSeconds(timer, 0)).toBe(300);
  });

  it("freezes the remaining duration while a bounded timer is paused", () => {
    const timer: ActiveTimer = {
      task_id: 2,
      started_at: Math.floor(Date.now() / 1000),
      elapsed_seconds: 0,
      is_running: false,
      timer_limit_seconds: 1500,
      timer_remaining_seconds: 840,
    };

    vi.advanceTimersByTime(60_000);

    expect(calculateTimerRemainingSeconds(timer)).toBe(840);
    expect(calculateDisplayElapsedSeconds(timer, 0)).toBe(660);
  });

  it("never reports negative remaining time after expiry", () => {
    const timer: ActiveTimer = {
      task_id: 2,
      started_at: Math.floor(Date.now() / 1000) - 60,
      elapsed_seconds: 0,
      is_running: true,
      timer_limit_seconds: 60,
      timer_remaining_seconds: 60,
      timer_expires_at: Math.floor(Date.now() / 1000) - 1,
    };

    expect(calculateTimerRemainingSeconds(timer)).toBe(0);
    expect(calculateDisplayElapsedSeconds(timer, 0)).toBe(60);
  });
});
