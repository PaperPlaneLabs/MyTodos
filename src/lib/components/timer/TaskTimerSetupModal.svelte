<script lang="ts">
  import Modal from "$lib/components/common/Modal.svelte";

  let {
    open,
    taskTitle,
    activeTaskTitle = null,
    onClose,
    onStart,
  }: {
    open: boolean;
    taskTitle: string;
    activeTaskTitle?: string | null;
    onClose: () => void;
    onStart: (durationSeconds: number) => Promise<void>;
  } = $props();

  const presets = [15, 25, 30, 45, 60];
  let selectedMinutes = $state(25);
  let customMinutes = $state("");
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let wasOpen = false;

  const durationMinutes = $derived(
    customMinutes.trim() ? Number(customMinutes) : selectedMinutes,
  );
  const isValid = $derived(
    Number.isFinite(durationMinutes) &&
      durationMinutes >= 1 &&
      durationMinutes <= 1440,
  );

  $effect(() => {
    if (open && !wasOpen) {
      selectedMinutes = 25;
      customMinutes = "";
      error = null;
    }
    wasOpen = open;
  });

  function selectPreset(minutes: number) {
    selectedMinutes = minutes;
    customMinutes = "";
    error = null;
  }

  async function submit() {
    if (!isValid || submitting) return;

    submitting = true;
    error = null;
    try {
      await onStart(Math.round(durationMinutes * 60));
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      submitting = false;
    }
  }
</script>

<Modal {open} title="Set Task Timer" {onClose}>
  {#snippet children()}
    <form
      class="timer-form"
      onsubmit={async (event) => {
        event.preventDefault();
        await submit();
      }}
    >
      <div class="task-name">{taskTitle}</div>
      <p class="helper">Choose how long you want to focus on this task.</p>

      <div class="presets" aria-label="Timer duration presets">
        {#each presets as minutes}
          <button
            type="button"
            class:active={!customMinutes && selectedMinutes === minutes}
            onclick={() => selectPreset(minutes)}
          >
            {minutes} min
          </button>
        {/each}
      </div>

      <label for="custom-task-timer" class="custom-label">Custom minutes</label>
      <input
        id="custom-task-timer"
        class="input"
        type="number"
        min="1"
        max="1440"
        step="1"
        bind:value={customMinutes}
        placeholder="For example, 90"
        oninput={() => (error = null)}
      />

      {#if activeTaskTitle}
        <p class="active-warning">
          The active timer for <strong>{activeTaskTitle}</strong> will be stopped and
          recorded first.
        </p>
      {/if}

      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-secondary" onclick={onClose}>Cancel</button>
        <button type="submit" class="btn btn-primary" disabled={!isValid || submitting}>
          {submitting ? "Starting…" : activeTaskTitle ? "Stop & Start Timer" : "Start Timer"}
        </button>
      </div>
    </form>
  {/snippet}
</Modal>

<style>
  .timer-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .task-name {
    font-weight: 650;
    color: var(--text-primary);
    overflow-wrap: anywhere;
  }

  .helper,
  .custom-label {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--spacing-sm);
  }

  .presets button {
    padding: 8px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-secondary);
    color: var(--text-primary);
    cursor: pointer;
  }

  .presets button.active {
    border-color: var(--accent);
    background: var(--accent-light);
    color: var(--accent);
  }

  .active-warning {
    padding: var(--spacing-sm);
    border: 1px solid var(--warning);
    border-radius: var(--radius-md);
    background: var(--warning-light);
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.45;
  }

  .error {
    color: var(--danger);
    font-size: 12px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }
</style>
