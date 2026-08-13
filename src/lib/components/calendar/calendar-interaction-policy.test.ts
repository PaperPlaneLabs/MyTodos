import { describe, expect, it } from "vitest";

import {
  shouldSelectWeekGridTarget,
  shouldShowBottomTimer,
} from "$lib/components/calendar/calendar-interaction-policy";

describe("calendar interaction policy", () => {
  it("shows the bottom timer only in Projects", () => {
    expect(shouldShowBottomTimer("projects")).toBe(true);
    expect(shouldShowBottomTimer("today")).toBe(false);
    expect(shouldShowBottomTimer("calendar")).toBe(false);
    expect(shouldShowBottomTimer("stats")).toBe(false);
    expect(shouldShowBottomTimer("settings")).toBe(false);
  });

  it("selects a week slot without treating an item click as a day click", () => {
    const lane = document.createElement("div");
    const item = document.createElement("button");
    item.className = "timed-item";
    const itemTitle = document.createElement("span");
    item.append(itemTitle);
    lane.append(item);

    expect(shouldSelectWeekGridTarget(lane)).toBe(true);
    expect(shouldSelectWeekGridTarget(itemTitle)).toBe(false);
  });
});
