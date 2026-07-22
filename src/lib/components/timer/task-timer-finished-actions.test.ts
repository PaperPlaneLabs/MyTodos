import { describe, expect, it, vi } from "vitest";

import {
  continueWithoutTimer,
  switchTask,
} from "$lib/components/timer/task-timer-finished-actions";

describe("task timer finished actions", () => {
  it("starts an ordinary timer before syncing and closing", async () => {
    const calls: string[] = [];
    const startTimer = vi.fn(async (taskId: number) => {
      calls.push(`start:${taskId}`);
    });

    await continueWithoutTimer(42, {
      startTimer,
      notifyTimerContinued: async () => {
        calls.push("notify");
      },
      closeWindow: async () => {
        calls.push("close");
      },
    });

    expect(startTimer).toHaveBeenCalledWith(42);
    expect(calls).toEqual(["start:42", "notify", "close"]);
  });

  it("focuses the main window before closing the completion window", async () => {
    const calls: string[] = [];

    await switchTask({
      notifySwitchRequested: async () => {
        calls.push("notify");
      },
      focusMainWindow: async () => {
        calls.push("focus");
      },
      closeWindow: async () => {
        calls.push("close");
      },
    });

    expect(calls).toEqual(["notify", "focus", "close"]);
  });
});
