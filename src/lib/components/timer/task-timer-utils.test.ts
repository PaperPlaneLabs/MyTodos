import { describe, expect, it } from "vitest";

import { resolveTaskTimerMinutes } from "./task-timer-utils";

describe("resolveTaskTimerMinutes", () => {
  it("uses a numeric custom duration from a number input", () => {
    expect(resolveTaskTimerMinutes(1, 25)).toBe(1);
  });

  it("falls back to the selected preset when custom input is empty", () => {
    expect(resolveTaskTimerMinutes(undefined, 25)).toBe(25);
  });
});
