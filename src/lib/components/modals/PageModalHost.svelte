<script lang="ts">
  import DateTimePicker from "$lib/components/common/DateTimePicker.svelte";
  import Modal from "$lib/components/common/Modal.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { taskStore } from "$lib/stores/tasks.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";

  let projectName = $state("");
  let taskTitle = $state("");
  let taskDeadline = $state<string | null>(null);
  let taskTime = $state("");
  let showResetModal = $state(false);
  let taskToReset = $state<number | null>(null);
  let showDeleteModal = $state(false);
  let itemToDelete = $state<{ type: "project" | "task"; id: number } | null>(
    null,
  );
  let lastEditingProjectId = $state<number | null>(null);
  let lastTaskModalKey = $state<string | null>(null);

  const isCalendarPresetDeadline = $derived(
    uiStore.showTaskModal &&
      !uiStore.editingTaskId &&
      !!uiStore.newTaskDeadline,
  );

  export function openResetModal(taskId: number) {
    taskToReset = taskId;
    showResetModal = true;
  }

  export function confirmDelete(type: "project" | "task", id: number) {
    itemToDelete = { type, id };
    showDeleteModal = true;
  }

  function formatPresetDeadline(deadline: string | null): string {
    if (!deadline) return "";
    const date = new Date(`${deadline}T00:00:00`);
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  $effect(() => {
    if (
      uiStore.showProjectModal &&
      uiStore.editingProjectId !== lastEditingProjectId
    ) {
      lastEditingProjectId = uiStore.editingProjectId;
      if (uiStore.editingProjectId) {
        const project = projectStore.projects.find(
          (item) => item.id === uiStore.editingProjectId,
        );
        projectName = project?.name ?? "";
      } else {
        projectName = "";
      }
    } else if (!uiStore.showProjectModal) {
      lastEditingProjectId = null;
    }
  });

  $effect(() => {
    if (uiStore.showTaskModal) {
      const modalKey = `${uiStore.editingTaskId ?? "new"}:${uiStore.newTaskDeadline ?? ""}`;
      if (modalKey === lastTaskModalKey) {
        return;
      }
      lastTaskModalKey = modalKey;

      if (uiStore.editingTaskId) {
        const task = taskStore.tasks.find(
          (item) => item.id === uiStore.editingTaskId,
        );
        if (task) {
          taskTitle = task.title;
          if (task.deadline) {
            const [date, time] = task.deadline.split("T");
            taskDeadline = date;
            taskTime = time ? time.substring(0, 5) : "";
          } else {
            taskDeadline = null;
            taskTime = "";
          }
        }
      } else {
        taskTitle = "";
        taskDeadline = uiStore.newTaskDeadline;
        taskTime = "";
      }
    } else {
      lastTaskModalKey = null;
      taskTitle = "";
      taskDeadline = null;
      taskTime = "";
    }
  });

  async function handleProjectSubmit() {
    if (!projectName.trim()) return;

    try {
      if (uiStore.editingProjectId) {
        await projectStore.update(uiStore.editingProjectId, projectName);
      } else {
        await projectStore.create(projectName);
      }

      projectName = "";
      uiStore.closeProjectModal();
    } catch (error) {
      console.error("Error saving project:", error);
    }
  }

  async function handleTaskSubmit() {
    if (!taskTitle.trim()) return;

    try {
      if (uiStore.editingTaskId) {
        await taskStore.updateTask(uiStore.editingTaskId, taskTitle);
        const fullDeadline = taskDeadline
          ? taskTime
            ? `${taskDeadline}T${taskTime}`
            : taskDeadline
          : null;
        await taskStore.updateDeadline(uiStore.editingTaskId, fullDeadline);
        uiStore.closeTaskModal();
        return;
      }

      const task = await taskStore.createTask(
        projectStore.selectedId,
        null,
        taskTitle,
      );
      if (taskDeadline) {
        const fullDeadline = taskTime
          ? `${taskDeadline}T${taskTime}`
          : taskDeadline;
        await taskStore.updateDeadline(task.id, fullDeadline);
      }

      taskTitle = "";
      taskDeadline = null;
      taskTime = "";
      uiStore.closeTaskModal();
    } catch (error) {
      console.error("Error saving task:", error);
    }
  }

  async function handleResetTimer() {
    if (taskToReset === null) return;

    if (timerStore.active && timerStore.active.task_id === taskToReset) {
      await timerStore.reset();
    } else {
      await taskStore.resetTaskTime(taskToReset);
      await projectStore.loadAll();
    }

    showResetModal = false;
    taskToReset = null;
  }

  async function handleDelete() {
    if (!itemToDelete) return;

    try {
      if (itemToDelete.type === "project") {
        await projectStore.delete(itemToDelete.id);
      } else {
        if (timerStore.active && timerStore.active.task_id === itemToDelete.id) {
          await timerStore.stop();
        }
        await taskStore.deleteTask(itemToDelete.id);
      }
    } catch (error) {
      console.error(`Failed to delete ${itemToDelete.type}:`, error);
    }

    showDeleteModal = false;
    itemToDelete = null;
  }
</script>

<Modal
  open={uiStore.showProjectModal}
  title={uiStore.editingProjectId ? "Edit Project" : "New Project"}
  onClose={() => uiStore.closeProjectModal()}
>
  {#snippet children()}
    <form
      onsubmit={async (event) => {
        event.preventDefault();
        await handleProjectSubmit();
      }}
    >
      <div class="form-stack">
        <div>
          <label for="project-name" class="text-sm text-secondary"
            >Project Name</label
          >
          <input
            id="project-name"
            class="input"
            type="text"
            bind:value={projectName}
            placeholder="My Project"
          />
        </div>
        <div class="form-actions">
          <button
            type="button"
            class="btn btn-secondary"
            onclick={() => uiStore.closeProjectModal()}>Cancel</button
          >
          <button type="submit" class="btn btn-primary"
            >{uiStore.editingProjectId ? "Save" : "Create"}</button
          >
        </div>
      </div>
    </form>
  {/snippet}
</Modal>

<Modal
  open={uiStore.showTaskModal}
  title={uiStore.editingTaskId ? "Edit Task" : "New Task"}
  onClose={() => uiStore.closeTaskModal()}
  allowOverflow={true}
>
  {#snippet children()}
    <form
      onsubmit={async (event) => {
        event.preventDefault();
        await handleTaskSubmit();
      }}
    >
      <div class="form-stack">
        <div>
          <label for="task-title" class="text-sm text-secondary">Task Title</label>
          <input
            id="task-title"
            class="input"
            type="text"
            bind:value={taskTitle}
            placeholder="Task title"
          />
        </div>

        <div>
          {#if isCalendarPresetDeadline}
            <div class="text-sm text-secondary">Deadline</div>
            <div class="deadline-input">
              <div class="deadline-fixed">
                <span>{formatPresetDeadline(taskDeadline)}</span>
              </div>
              <input
                type="time"
                class="input"
                bind:value={taskTime}
                step="300"
                style="width: 110px;"
              />
            </div>
          {:else}
            <label for="task-deadline" class="text-sm text-secondary"
              >Deadline (optional)</label
            >
            <DateTimePicker
              bind:date={taskDeadline}
              bind:time={taskTime}
              triggerAriaLabel="Choose an optional task deadline"
            />
          {/if}
        </div>

        <div class="form-actions">
          <button
            type="button"
            class="btn btn-secondary"
            onclick={() => uiStore.closeTaskModal()}>Cancel</button
          >
          <button type="submit" class="btn btn-primary"
            >{uiStore.editingTaskId ? "Save" : "Create"}</button
          >
        </div>
      </div>
    </form>
  {/snippet}
</Modal>

<Modal
  open={showResetModal}
  title="⚠️ Reset Timer"
  onClose={() => (showResetModal = false)}
>
  {#snippet children()}
    <div class="reset-modal-content">
      <div class="reset-warning">
        <div class="warning-icon">⚠️</div>
        <div class="warning-text">
          <p class="warning-title">Are you sure you want to reset the timer?</p>
          <p class="warning-description">
            All time tracking for this session will be permanently lost.
          </p>
        </div>
      </div>
      <div class="reset-actions">
        <button
          type="button"
          class="btn btn-secondary"
          onclick={() => (showResetModal = false)}>Cancel</button
        >
        <button
          type="button"
          class="btn btn-warning"
          onclick={handleResetTimer}>Reset Timer</button
        >
      </div>
    </div>
  {/snippet}
</Modal>

<Modal
  open={showDeleteModal}
  title="⚠️ Delete {itemToDelete?.type === 'project' ? 'Project' : 'Task'}"
  onClose={() => (showDeleteModal = false)}
>
  {#snippet children()}
    <div class="reset-modal-content">
      <div class="delete-warning reset-warning">
        <div class="warning-icon">🗑️</div>
        <div class="warning-text">
          <p class="warning-title">Delete this {itemToDelete?.type}?</p>
          <p class="warning-description">
            This action cannot be undone. All associated data will be lost.
          </p>
        </div>
      </div>
      <div class="reset-actions">
        <button
          type="button"
          class="btn btn-secondary"
          onclick={() => (showDeleteModal = false)}>Cancel</button
        >
        <button type="button" class="btn btn-danger" onclick={handleDelete}
          >Delete</button
        >
      </div>
    </div>
  {/snippet}
</Modal>

<style>
  .form-stack {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .form-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
  }

  input[type="time"] {
    appearance: none;
    -webkit-appearance: none;
    font-family: var(--font-mono);
  }

  :global([data-theme="dark"]) input[type="time"],
  :global([data-theme="retro"]) input[type="time"],
  :global([data-theme="ocean"]) input[type="time"],
  :global([data-theme="nord"]) input[type="time"],
  :global([data-theme="minecraft"]) input[type="time"] {
    color-scheme: dark;
  }

  input[type="time"]::-webkit-calendar-picker-indicator {
    cursor: pointer;
    opacity: 0.5;
    transition: all 0.2s;
    filter: invert(0);
  }

  :global([data-theme="dark"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="retro"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="ocean"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="nord"])
    input[type="time"]::-webkit-calendar-picker-indicator,
  :global([data-theme="minecraft"])
    input[type="time"]::-webkit-calendar-picker-indicator {
    filter: invert(1);
    opacity: 0.7;
  }

  input[type="time"]::-webkit-calendar-picker-indicator:hover {
    opacity: 1;
    transform: scale(1.1);
  }

  .deadline-input {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .deadline-fixed {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: var(--text-sm);
  }

  .reset-modal-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
  }

  .reset-warning {
    display: flex;
    gap: var(--spacing-md);
    padding: var(--spacing-md);
    background: linear-gradient(
      135deg,
      var(--warning-light) 0%,
      var(--warning-gradient-end) 100%
    );
    border: 2px solid var(--warning);
    border-radius: var(--radius-md);
  }

  .delete-warning {
    background: linear-gradient(
      135deg,
      var(--danger-light) 0%,
      var(--danger-glow) 100%
    );
    border-color: var(--danger);
  }

  .warning-icon {
    font-size: 32px;
    flex-shrink: 0;
  }

  .warning-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .warning-title {
    font-weight: 600;
    font-size: 15px;
    color: var(--text-primary);
    margin: 0;
  }

  .warning-description {
    font-size: 13px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .reset-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
  }
</style>
