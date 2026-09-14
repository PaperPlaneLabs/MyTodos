import { tick } from "svelte";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import PageModalHost from "$lib/components/modals/PageModalHost.svelte";

const state = vi.hoisted(() => ({
  ui: {
    showProjectModal: false,
    showTaskModal: true,
    editingProjectId: null as number | null,
    editingTaskId: null as number | null,
    newTaskDeadline: "2026-08-12" as string | null,
    closeProjectModal: vi.fn(),
    closeTaskModal: vi.fn(),
  },
  projects: {
    projects: [
      { id: 1, name: "Personal" },
      { id: 2, name: "Work" },
    ],
    selectedId: 1 as number | null,
    create: vi.fn(),
    update: vi.fn(),
    loadAll: vi.fn(),
  },
  tasks: {
    tasks: [] as Array<Record<string, unknown>>,
    createTask: vi.fn().mockResolvedValue({ id: 31 }),
    updateTask: vi.fn(),
    updateDeadline: vi.fn().mockResolvedValue(undefined),
    resetTaskTime: vi.fn(),
  },
  timer: { active: null, reset: vi.fn() },
}));

vi.mock("$lib/stores/ui.svelte", () => ({ uiStore: state.ui }));
vi.mock("$lib/stores/projects.svelte", () => ({ projectStore: state.projects }));
vi.mock("$lib/stores/tasks.svelte", () => ({ taskStore: state.tasks }));
vi.mock("$lib/stores/timer.svelte", () => ({ timerStore: state.timer }));

afterEach(cleanup);

beforeEach(() => {
  state.ui.showTaskModal = true;
  state.ui.editingTaskId = null;
  state.ui.newTaskDeadline = "2026-08-12";
  state.projects.selectedId = 1;
  state.tasks.createTask.mockClear();
  state.tasks.updateDeadline.mockClear();
  state.ui.closeTaskModal.mockClear();
});

describe("PageModalHost task form", () => {
  it("keeps a preset date editable and assigns a new task to the selected project", async () => {
    const { container } = render(PageModalHost);

    expect(screen.getByRole("button", { name: /Deadline: Aug 12, 2026/i })).toBeTruthy();
    const project = screen.getByLabelText("Project") as HTMLSelectElement;
    expect(project.value).toBe("1");

    await fireEvent.input(screen.getByLabelText("Task Title"), { target: { value: "Prepare review" } });
    await fireEvent.change(project, { target: { value: "2" } });
    await tick();
    expect(project.value).toBe("2");
    await fireEvent.submit(container.querySelector("form")!);

    expect(state.tasks.createTask).toHaveBeenCalledWith(2, null, "Prepare review");
    expect(state.tasks.updateDeadline).toHaveBeenCalledWith(31, "2026-08-12");
  });
});
