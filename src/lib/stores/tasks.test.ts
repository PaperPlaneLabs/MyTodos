import { beforeEach, describe, expect, it, vi } from "vitest";

const mocked = vi.hoisted(() => ({
  create: vi.fn(),
  getByProject: vi.fn(),
  getUnassigned: vi.fn(),
  refreshCalendar: vi.fn(),
  refreshToday: vi.fn(),
}));

vi.mock("$lib/services/db", () => ({
  db: {
    tasks: {
      create: mocked.create,
      getByProject: mocked.getByProject,
      getUnassigned: mocked.getUnassigned,
    },
  },
}));
vi.mock("$lib/stores/calendar.svelte", () => ({ calendarStore: { refreshCurrentRange: mocked.refreshCalendar } }));
vi.mock("$lib/stores/today.svelte", () => ({ todayStore: { refresh: mocked.refreshToday } }));

import { taskStore } from "$lib/stores/tasks.svelte";

beforeEach(() => {
  mocked.getByProject.mockResolvedValue([]);
  mocked.getUnassigned.mockResolvedValue([]);
  mocked.create.mockResolvedValue({ id: 5, title: "Elsewhere", completed: false, position: 0, total_time_seconds: 0 });
  mocked.refreshCalendar.mockResolvedValue(undefined);
  mocked.refreshToday.mockResolvedValue(undefined);
});

describe("taskStore.createTask", () => {
  it("does not insert a task assigned to another project into the visible task list", async () => {
    await taskStore.loadByProject(1);

    await taskStore.createTask(2, null, "Elsewhere");

    expect(taskStore.tasks).toEqual([]);
  });
});
