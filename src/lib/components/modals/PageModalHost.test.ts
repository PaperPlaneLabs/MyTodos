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
    editingTask: null as Record<string, unknown> | null,
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
    getTask: vi.fn().mockResolvedValue(null),
    createTask: vi.fn().mockResolvedValue({ id: 31 }),
    updateTask: vi.fn().mockResolvedValue(undefined),
    moveTask: vi.fn().mockResolvedValue(undefined),
    updateDeadline: vi.fn().mockResolvedValue(undefined),
    resetTaskTime: vi.fn(),
  },
  today: {
    taskSummary: {
      today: [] as Array<Record<string, unknown>>,
      upcoming: [] as Array<Record<string, unknown>>,
      overdue: [] as Array<Record<string, unknown>>,
      completed_today: 0,
      total_today: 0,
    },
    findTask: vi.fn().mockReturnValue(undefined),
  },
  timer: { active: null, reset: vi.fn() },
}));

vi.mock("$lib/stores/ui.svelte", () => ({ uiStore: state.ui }));
vi.mock("$lib/stores/projects.svelte", () => ({ projectStore: state.projects }));
vi.mock("$lib/stores/tasks.svelte", () => ({ taskStore: state.tasks }));
vi.mock("$lib/stores/today.svelte", () => ({ todayStore: state.today }));
vi.mock("$lib/stores/timer.svelte", () => ({ timerStore: state.timer }));

afterEach(cleanup);

beforeEach(() => {
  state.ui.showTaskModal = true;
  state.ui.editingTaskId = null;
  state.ui.editingTask = null;
  state.ui.newTaskDeadline = "2026-08-12";
  state.projects.selectedId = 1;
  state.tasks.tasks = [];
  state.tasks.getTask.mockClear();
  state.tasks.createTask.mockClear();
  state.tasks.updateTask.mockClear();
  state.tasks.moveTask.mockClear();
  state.tasks.updateDeadline.mockClear();
  state.ui.closeTaskModal.mockClear();
  state.today.findTask.mockClear();
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

  it("populates task details from editingTask when editing from dashboard and allows changing project and deadline", async () => {
    state.ui.editingTaskId = 42;
    state.ui.editingTask = {
      id: 42,
      title: "Dashboard Upcoming Task",
      deadline: "2026-08-16T14:30:00",
      project_id: 1,
    };

    const { container } = render(PageModalHost);

    const titleInput = screen.getByLabelText("Task Title") as HTMLInputElement;
    expect(titleInput.value).toBe("Dashboard Upcoming Task");

    const projectSelect = screen.getByLabelText("Project") as HTMLSelectElement;
    expect(projectSelect.value).toBe("1");

    // Change the project from 1 (Personal) to 2 (Work)
    await fireEvent.change(projectSelect, { target: { value: "2" } });
    await tick();
    expect(projectSelect.value).toBe("2");

    // Submit the form
    await fireEvent.submit(container.querySelector("form")!);
    await tick();
    await new Promise((resolve) => setTimeout(resolve, 0));

    // Should move task to project 2
    expect(state.tasks.moveTask).toHaveBeenCalledWith(42, 2);
    // Should update title
    expect(state.tasks.updateTask).toHaveBeenCalledWith(42, "Dashboard Upcoming Task");
    // Should update deadline
    expect(state.tasks.updateDeadline).toHaveBeenCalledWith(42, "2026-08-16T14:30");
    // Should close modal
    expect(state.ui.closeTaskModal).toHaveBeenCalled();
  });

  it("fetches task asynchronously via taskStore.getTask if not found in memory", async () => {
    state.ui.editingTaskId = 99;
    state.ui.editingTask = null;
    state.tasks.getTask.mockResolvedValue({
      id: 99,
      title: "DB Task",
      deadline: "2026-08-18",
      project_id: 2,
    });

    render(PageModalHost);

    await tick();
    // Wait for the promise to resolve
    await new Promise((resolve) => setTimeout(resolve, 0));
    await tick();

    const titleInput = screen.getByLabelText("Task Title") as HTMLInputElement;
    expect(titleInput.value).toBe("DB Task");

    const projectSelect = screen.getByLabelText("Project") as HTMLSelectElement;
    expect(projectSelect.value).toBe("2");
  });
});
