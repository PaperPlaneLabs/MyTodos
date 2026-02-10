import type { Task } from "$lib/services/db";
import type { TimeEntryWithTask } from "$lib/types/calendar";

let showProjectModal = $state(false);
let showTaskModal = $state(false);
let showSectionModal = $state(false);
let showManualTimeModal = $state(false);
let showQuickAdd = $state(false);
let showStatsView = $state(false);
let showSettingsView = $state(false);
let showCalendarView = $state(false);
let isCollapsed = $state(false);
let completedTasksCollapsed = $state(true);
let handleTop = $state(120);
let editingProjectId = $state<number | null>(null);
let editingTaskId = $state<number | null>(null);
let editingSectionId = $state<number | null>(null);
let manualTimeTaskId = $state<number | null>(null);
let newTaskDeadline = $state<string | null>(null);
export type Theme = "light" | "dark" | "minecraft" | "retro" | "ocean" | "nord";
let theme = $state<Theme>("light");
export type WindowOrientation = "left" | "right" | "center";
let windowOrientation = $state<WindowOrientation>("center");
let calendarSelectedEntry = $state<TimeEntryWithTask | null>(null);
type TaskModalPayload = number | { taskId?: number; task?: Pick<Task, "id">; deadline?: string };

// Context Menu State
let contextMenuOpen = $state(false);
let contextMenuPos = $state({ x: 0, y: 0 });
let contextMenuType = $state<"project" | "task" | null>(null);
let contextMenuId = $state<number | null>(null);

export const uiStore = {
  get showProjectModal() {
    return showProjectModal;
  },

  get showTaskModal() {
    return showTaskModal;
  },

  get showSectionModal() {
    return showSectionModal;
  },

  get showManualTimeModal() {
    return showManualTimeModal;
  },

  get editingProjectId() {
    return editingProjectId;
  },

  get editingTaskId() {
    return editingTaskId;
  },

  get editingSectionId() {
    return editingSectionId;
  },

  get manualTimeTaskId() {
    return manualTimeTaskId;
  },

  get newTaskDeadline() {
    return newTaskDeadline;
  },

  get showQuickAdd() {
    return showQuickAdd;
  },

  get showStatsView() {
    return showStatsView;
  },

  get showSettingsView() {
    return showSettingsView;
  },

  get showCalendarView() {
    return showCalendarView;
  },

  get windowOrientation() {
    return windowOrientation;
  },

  get calendarSelectedEntry() {
    return calendarSelectedEntry;
  },

  get theme() {
    return theme;
  },

  get isCollapsed() {
    return isCollapsed;
  },

  get handleTop() {
    return handleTop;
  },

  get completedTasksCollapsed() {
    return completedTasksCollapsed;
  },

  // Context Menu Getters
  get contextMenuOpen() { return contextMenuOpen; },
  get contextMenuPos() { return contextMenuPos; },
  get contextMenuType() { return contextMenuType; },
  get contextMenuId() { return contextMenuId; },

  toggleQuickAdd() {
    showQuickAdd = !showQuickAdd;
  },

  closeQuickAdd() {
    showQuickAdd = false;
  },

  setCollapsed(collapsed: boolean) {
    isCollapsed = collapsed;
  },

  setHandleTop(top: number) {
    handleTop = top;
  },

  toggleCompletedTasks() {
    completedTasksCollapsed = !completedTasksCollapsed;
  },

  openStatsView() {
    showStatsView = true;
  },

  closeStatsView() {
    showStatsView = false;
  },

  openSettingsView() {
    showSettingsView = true;
  },

  closeSettingsView() {
    showSettingsView = false;
  },

  openCalendarView() {
    showCalendarView = true;
  },

  closeCalendarView() {
    showCalendarView = false;
    calendarSelectedEntry = null;
  },

  setWindowOrientation(orientation: WindowOrientation) {
    windowOrientation = orientation;
  },

  selectCalendarEntry(entry: TimeEntryWithTask | null) {
    calendarSelectedEntry = entry;
  },

  openProjectModal(projectId: number | null = null) {
    editingProjectId = projectId;
    showProjectModal = true;
  },

  closeProjectModal() {
    showProjectModal = false;
    editingProjectId = null;
  },

  openTaskModal(payload: TaskModalPayload = {}) {
    const data = typeof payload === "number" ? { taskId: payload } : payload;
    editingTaskId = data.taskId ?? data.task?.id ?? null;
    newTaskDeadline = data.deadline ?? null;
    showTaskModal = true;
  },

  closeTaskModal() {
    showTaskModal = false;
    editingTaskId = null;
    newTaskDeadline = null;
  },

  openSectionModal(sectionId: number | null = null) {
    editingSectionId = sectionId;
    showSectionModal = true;
  },

  closeSectionModal() {
    showSectionModal = false;
    editingSectionId = null;
  },

  openManualTimeModal(taskId: number) {
    manualTimeTaskId = taskId;
    showManualTimeModal = true;
  },

  closeManualTimeModal() {
    showManualTimeModal = false;
    manualTimeTaskId = null;
  },

  // Context Menu Methods
  openContextMenu(x: number, y: number, type: "project" | "task", id: number) {
    contextMenuPos = { x, y };
    contextMenuType = type;
    contextMenuId = id;
    contextMenuOpen = true;
  },

  closeContextMenu() {
    contextMenuOpen = false;
  },

  setTheme(newTheme: Theme) {
    theme = newTheme;
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
  },

  initTheme() {
    const savedTheme = localStorage.getItem("theme") as "light" | "dark" | null;
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

    theme = (savedTheme as Theme) ?? (prefersDark ? "dark" : "light");
    document.documentElement.setAttribute("data-theme", theme);
  },
};
