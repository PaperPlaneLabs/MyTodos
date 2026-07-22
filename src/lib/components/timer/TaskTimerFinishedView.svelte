<script lang="ts">
  import { onMount } from "svelte";
  import { fade, scale } from "svelte/transition";

  import {
    continueWithoutTimer as runContinueWithoutTimer,
    switchTask as runSwitchTask,
  } from "$lib/components/timer/task-timer-finished-actions";
  import { db } from "$lib/services/db";

  interface CompletionData {
    taskId: number;
    taskTitle: string;
    durationSeconds: number;
    finishedAt: number;
  }

  let completion = $state<CompletionData>({
    taskId: 0,
    taskTitle: "Task",
    durationSeconds: 0,
    finishedAt: 0,
  });
  let sending = $state(false);
  let mounted = $state(false);

  onMount(() => {
    document.documentElement.setAttribute(
      "data-theme",
      localStorage.getItem("theme") ?? "light",
    );
    if (window.__TASK_TIMER_FINISHED__) {
      completion = window.__TASK_TIMER_FINISHED__;
    }

    let unlisten: (() => void) | undefined;
    void import("@tauri-apps/api/event").then(async ({ listen }) => {
      unlisten = await listen<CompletionData>(
        "task-timer-finished:update",
        (event) => (completion = event.payload),
      );
    });

    requestAnimationFrame(() => (mounted = true));
    return () => unlisten?.();
  });

  function formatDuration(seconds: number): string {
    const minutes = Math.round(seconds / 60);
    if (minutes < 60) return `${minutes} minute${minutes === 1 ? "" : "s"}`;
    const hours = Math.floor(minutes / 60);
    const remainder = minutes % 60;
    return remainder === 0
      ? `${hours} hour${hours === 1 ? "" : "s"}`
      : `${hours}h ${remainder}m`;
  }

  async function continueWithoutTimer() {
    if (sending) return;
    sending = true;
    try {
      const { emit } = await import("@tauri-apps/api/event");
      await runContinueWithoutTimer(completion.taskId, {
        startTimer: async (taskId) => {
          await db.timer.start(taskId);
        },
        notifyTimerContinued: () => emit("timer:continued"),
        closeWindow: db.window.closeTaskTimerFinished,
      });
    } catch (error) {
      console.error("Failed to continue task without a timer:", error);
      sending = false;
    }
  }

  async function closeWindow() {
    if (sending) return;
    sending = true;
    try {
      await db.window.closeTaskTimerFinished();
    } catch (error) {
      console.error("Failed to close task timer window:", error);
      sending = false;
    }
  }

  async function switchTask() {
    if (sending) return;
    sending = true;
    try {
      const { emit } = await import("@tauri-apps/api/event");
      await runSwitchTask({
        notifySwitchRequested: () => emit("task-timer:switch-requested"),
        focusMainWindow: db.window.focusMain,
        closeWindow: db.window.closeTaskTimerFinished,
      });
    } catch (error) {
      console.error("Failed to switch task:", error);
      sending = false;
    }
  }

  async function dragWindow() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("start_window_drag");
    } catch {
      // Dragging is optional; keep the actions usable if it is unavailable.
    }
  }
</script>

<div class="shell" class:mounted>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="drag-bar" onmousedown={dragWindow} role="presentation">
    <span>Task timer</span>
    <button class="close" aria-label="Close" onclick={closeWindow}>×</button>
  </div>

  <main class="content">
    <div class="success" in:scale={{ duration: 350 }}>✓</div>
    <div class="copy" in:fade={{ duration: 250, delay: 100 }}>
      <h1>Focus session complete</h1>
      <p>
        <strong>{formatDuration(completion.durationSeconds)}</strong> was added to
        <span>{completion.taskTitle}</span>.
      </p>
    </div>

    <div class="actions" in:fade={{ duration: 200, delay: 180 }}>
      <button class="btn btn-primary" disabled={sending} onclick={switchTask}>
        Switch task
      </button>
      <button
        class="btn btn-secondary"
        disabled={sending}
        onclick={continueWithoutTimer}
      >
        Continue without timer
      </button>
    </div>
  </main>
</div>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    overflow: hidden;
    background: transparent;
  }

  .shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--bg-primary);
    color: var(--text-primary);
    opacity: 0;
    transform: translateY(6px) scale(0.97);
    transition: opacity 180ms ease, transform 220ms ease;
  }

  .shell.mounted {
    opacity: 1;
    transform: none;
  }

  .drag-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 9px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 650;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    cursor: grab;
    user-select: none;
  }

  .close {
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--text-secondary);
    font-size: 19px;
    cursor: pointer;
  }

  .close:hover {
    background: var(--danger-light);
    color: var(--danger);
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 18px;
    padding: 24px 30px 30px;
    text-align: center;
  }

  .success {
    display: grid;
    place-items: center;
    width: 68px;
    height: 68px;
    border-radius: 50%;
    background: var(--success-light);
    color: var(--success);
    font-size: 34px;
    font-weight: 800;
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  h1 {
    margin: 0;
    font-size: 20px;
  }

  p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 13px;
    line-height: 1.55;
  }

  p span {
    display: block;
    margin-top: 3px;
    color: var(--text-primary);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .actions button {
    width: 100%;
  }
</style>
