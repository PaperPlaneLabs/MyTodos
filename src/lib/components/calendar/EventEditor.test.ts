import { cleanup, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import EventEditor from "$lib/components/calendar/EventEditor.svelte";

vi.mock("$lib/stores/calendar.svelte", () => ({
  calendarStore: {
    closeEventEditor: vi.fn(),
    saveEvent: vi.fn(),
  },
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    scaleFactor: vi.fn().mockResolvedValue(1),
    innerSize: vi.fn().mockResolvedValue({ toLogical: () => ({ width: 400, height: 900 }) }),
  }),
  LogicalSize: class LogicalSize {},
}));

afterEach(cleanup);

describe("EventEditor", () => {
  it("uses the shared picker for start, end, and recurrence end dates", () => {
    const { container } = render(EventEditor, {
      props: {
        draft: {
          id: null,
          title: "Planning",
          description: "",
          isAllDay: true,
          startDate: "2026-08-12",
          endDate: "2026-08-12",
          startTime: "09:00",
          endTime: "10:00",
          color: "#6366f1",
          recurrence: { frequency: "daily", interval: 1, weekdays: [], until: "2026-08-20", count: null },
        },
      },
    });

    expect(screen.getByRole("button", { name: /Event start date: Aug 12, 2026/i })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Event end date: Aug 12, 2026/i })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Recurrence end date: Aug 20, 2026/i })).toBeTruthy();
    expect(within(container).queryAllByDisplayValue("2026-08-12")).toHaveLength(0);
    expect(container.querySelectorAll('input[type="date"]')).toHaveLength(0);
  });
});
