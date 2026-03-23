import { projectStore } from "$lib/stores/projects.svelte";
import { taskStore } from "$lib/stores/tasks.svelte";
import { uiStore } from "$lib/stores/ui.svelte";

type ItemType = "project" | "task";

interface ColorPickerItem {
  type: "colorPicker";
  label: string;
  currentColor: string;
  onSelect: (color: string) => void;
}

interface ActionMenuItem {
  type?: "action" | "separator";
  label: string;
  icon?: string;
  onClick: () => void;
  danger?: boolean;
}

type ContextMenuItem = ActionMenuItem | ColorPickerItem;

interface CreatePageInteractionsOptions {
  onConfirmDelete: (type: ItemType, id: number) => void;
}

export function createPageInteractions({
  onConfirmDelete,
}: CreatePageInteractionsOptions) {
  let isDragging = $state(false);
  let dragType = $state<ItemType | null>(null);
  let draggedId = $state<number | null>(null);
  let currentIndex = $state<number | null>(null);
  let pointerId = $state<number | null>(null);
  let startY = $state(0);
  let startX = $state(0);
  let hasMovedThreshold = $state(false);
  let longPressTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  let contextMenuItems = $derived.by((): ContextMenuItem[] => {
    if (!uiStore.contextMenuType || uiStore.contextMenuId === null) {
      return [];
    }

    const id = uiStore.contextMenuId;

    if (uiStore.contextMenuType === "project") {
      const project = projectStore.projects.find((item) => item.id === id);
      return [
        {
          type: "colorPicker",
          label: "Color",
          currentColor: project?.color ?? "#6366f1",
          onSelect: (color: string) => projectStore.updateColor(id, color),
        },
        {
          label: "Edit Project",
          icon: "✏️",
          onClick: () => uiStore.openProjectModal(id),
        },
        {
          label: "Delete Project",
          icon: "🗑️",
          danger: true,
          onClick: () => onConfirmDelete("project", id),
        },
      ];
    }

    return [
      {
        label: "Edit Task",
        icon: "✏️",
        onClick: () => uiStore.openTaskModal(id),
      },
      {
        label: "Delete Task",
        icon: "🗑️",
        danger: true,
        onClick: () => onConfirmDelete("task", id),
      },
    ];
  });

  function handleContextMenu(
    event: MouseEvent | PointerEvent,
    type: ItemType,
    id: number,
  ) {
    event.preventDefault();
    uiStore.openContextMenu(event.clientX, event.clientY, type, id);
  }

  function handlePointerDown(
    event: PointerEvent,
    type: ItemType,
    id: number,
    index: number,
  ) {
    if (event.button === 2) return;

    pointerId = event.pointerId;
    dragType = type;
    draggedId = id;
    currentIndex = index;
    startY = event.clientY;
    startX = event.clientX;
    hasMovedThreshold = false;

    if (longPressTimer) clearTimeout(longPressTimer);
    longPressTimer = setTimeout(() => {
      if (!hasMovedThreshold) {
        uiStore.openContextMenu(startX, startY, type, id);
        cancelDrag();
      }
    }, 600);
  }

  function handlePointerMove(event: PointerEvent) {
    if (pointerId !== event.pointerId || draggedId === null) return;

    if (!hasMovedThreshold) {
      if (
        Math.abs(event.clientY - startY) > 5 ||
        Math.abs(event.clientX - startX) > 5
      ) {
        hasMovedThreshold = true;
        isDragging = true;
        if (longPressTimer) {
          clearTimeout(longPressTimer);
          longPressTimer = null;
        }
      } else {
        return;
      }
    }

    if (event.buttons !== 1) {
      void handlePointerUp(event);
      return;
    }

    const element = document.elementFromPoint(event.clientX, event.clientY);
    const wrapper = element?.closest(".draggable-wrapper, .task-item-wrapper");

    if (!wrapper) {
      return;
    }

    const hoveredType = wrapper.classList.contains("draggable-wrapper")
      ? "project"
      : "task";
    if (hoveredType !== dragType) {
      return;
    }

    const newIndex = parseInt(wrapper.getAttribute("data-index") || "-1");
    if (newIndex === -1 || newIndex === currentIndex) {
      return;
    }

    if (dragType === "project") {
      projectStore.reorderLocal(currentIndex!, newIndex);
    } else {
      taskStore.reorderLocal(currentIndex!, newIndex);
    }
    currentIndex = newIndex;
  }

  async function handlePointerUp(event: PointerEvent) {
    if (pointerId !== event.pointerId) return;

    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }

    if (isDragging) {
      try {
        if (dragType === "project") {
          await projectStore.reorder(projectStore.projects.map((item) => item.id));
        } else if (dragType === "task") {
          await taskStore.reorder(taskStore.tasks.map((item) => item.id));
        }
      } catch (error) {
        console.error("Failed to save order:", error);
      }
    }

    cancelDrag();
  }

  function cancelDrag() {
    if (longPressTimer) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }

    isDragging = false;
    dragType = null;
    draggedId = null;
    currentIndex = null;
    pointerId = null;
    hasMovedThreshold = false;
  }

  function handleKeySelect(event: KeyboardEvent, id: number | null) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      projectStore.setSelected(id);
    }
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      cancelDrag();
    }
  }

  function handleWindowClick() {
    uiStore.closeContextMenu();
  }

  return {
    get isDragging() {
      return isDragging;
    },
    get draggedId() {
      return draggedId;
    },
    get contextMenuItems() {
      return contextMenuItems;
    },
    handleContextMenu,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
    cancelDrag,
    handleKeySelect,
    handleWindowKeydown,
    handleWindowClick,
  };
}
