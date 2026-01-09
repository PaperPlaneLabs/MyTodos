let showProjectModal = $state(false);
let showTaskModal = $state(false);
let showSectionModal = $state(false);
let showManualTimeModal = $state(false);
let showQuickAdd = $state(false);
let isCollapsed = $state(false);
let handleTop = $state(120);
let editingProjectId = $state<number | null>(null);
let editingTaskId = $state<number | null>(null);
let editingSectionId = $state<number | null>(null);
let manualTimeTaskId = $state<number | null>(null);
let theme = $state<"light" | "dark">("light");

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

  get showQuickAdd() {
    return showQuickAdd;
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

  openProjectModal(projectId: number | null = null) {
    editingProjectId = projectId;
    showProjectModal = true;
  },

  closeProjectModal() {
    showProjectModal = false;
    editingProjectId = null;
  },

  openTaskModal(taskId: number | null = null) {
    editingTaskId = taskId;
    showTaskModal = true;
  },

  closeTaskModal() {
    showTaskModal = false;
    editingTaskId = null;
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

  toggleTheme() {
    theme = theme === "light" ? "dark" : "light";
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
  },

  initTheme() {
    const savedTheme = localStorage.getItem("theme") as "light" | "dark" | null;
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

    theme = savedTheme ?? (prefersDark ? "dark" : "light");
    document.documentElement.setAttribute("data-theme", theme);
  },
};
