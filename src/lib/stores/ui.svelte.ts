import type { Task } from "$lib/services/db";

let showProjectModal = $state(false);
let showTaskModal = $state(false);
let showQuickAdd = $state(false);
export type PrimaryView = "today" | "projects" | "calendar" | "stats" | "settings";
let primaryView = $state<PrimaryView>("today");
let isCollapsed = $state(false);
let completedTasksCollapsed = $state(true);
let handleTop = $state(120);
let editingProjectId = $state<number | null>(null);
let editingTaskId = $state<number | null>(null);
let newTaskDeadline = $state<string | null>(null);
export type Theme = "light" | "dark" | "minecraft" | "retro" | "ocean" | "nord";
let theme = $state<Theme>("light");
export type WindowOrientation = "left" | "right" | "center";
let windowOrientation = $state<WindowOrientation>("center");
let compactMode = $state(false);
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

  get editingProjectId() {
    return editingProjectId;
  },

  get editingTaskId() {
    return editingTaskId;
  },

  get newTaskDeadline() {
    return newTaskDeadline;
  },

  get showQuickAdd() {
    return showQuickAdd;
  },

  get showStatsView() {
    return primaryView === "stats";
  },

  get showSettingsView() {
    return primaryView === "settings";
  },

  get showCalendarView() {
    return primaryView === "calendar";
  },

  get primaryView() {
    return primaryView;
  },

  get windowOrientation() {
    return windowOrientation;
  },

  get theme() {
    return theme;
  },

  get compactMode() {
    return compactMode;
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
    primaryView = "stats";
  },

  closeStatsView() {
    primaryView = "projects";
  },

  openSettingsView() {
    primaryView = "settings";
  },

  closeSettingsView() {
    primaryView = "projects";
  },

  openCalendarView() {
    primaryView = "calendar";
  },

  closeCalendarView() {
    primaryView = "projects";
  },

  openTodayView() {
    primaryView = "today";
  },

  openProjectsView() {
    primaryView = "projects";
  },

  setPrimaryView(view: PrimaryView) {
    primaryView = view;
  },

  setWindowOrientation(orientation: WindowOrientation) {
    windowOrientation = orientation;
    // Note: The actual window orientation logic is handled in src-tauri/src/commands/window.rs
    // We assume there is a command to update this, or the frontend just stores the preference 
    // and the backend reads it or we invoke a command.
    // For now we just update the store.
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

  setCompactMode(enabled: boolean) {
    compactMode = enabled;
    if (enabled) {
      document.body.classList.add("compact-mode");
    } else {
      document.body.classList.remove("compact-mode");
    }
    localStorage.setItem("compactMode", String(enabled));
  },

  initTheme() {
    const savedTheme = localStorage.getItem("theme") as "light" | "dark" | null;
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

    theme = (savedTheme as Theme) ?? (prefersDark ? "dark" : "light");
    document.documentElement.setAttribute("data-theme", theme);

    // Init compact mode
    const savedCompact = localStorage.getItem("compactMode");
    compactMode = savedCompact === "true";
    if (compactMode) {
      document.body.classList.add("compact-mode");
    }
  },
};
