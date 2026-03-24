<script lang="ts">
  import { tick } from "svelte";
  import { fade, fly } from "svelte/transition";

  let {
    open = false,
    title,
    onClose,
    children,
    allowOverflow = false,
  }: {
    open: boolean;
    title: string;
    onClose: () => void;
    children: any;
    allowOverflow?: boolean;
  } = $props();

  const titleId = `modal-title-${Math.random().toString(36).slice(2, 10)}`;

  let dialogElement = $state<HTMLDivElement | null>(null);
  let closeButtonElement = $state<HTMLButtonElement | null>(null);
  let lastFocusedElement = $state<HTMLElement | null>(null);

  async function focusInitialControl() {
    await tick();
    closeButtonElement?.focus();
  }

  function getFocusableElements(): HTMLElement[] {
    if (!dialogElement) return [];

    return Array.from(
      dialogElement.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ),
    ).filter((element) => !element.hasAttribute("disabled"));
  }

  function handleDialogKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.stopPropagation();
      onClose();
      return;
    }

    if (event.key !== "Tab") return;

    const focusableElements = getFocusableElements();
    if (focusableElements.length === 0) {
      event.preventDefault();
      dialogElement?.focus();
      return;
    }

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];
    const activeElement = document.activeElement;

    if (event.shiftKey && activeElement === firstElement) {
      event.preventDefault();
      lastElement.focus();
    } else if (!event.shiftKey && activeElement === lastElement) {
      event.preventDefault();
      firstElement.focus();
    }
  }

  $effect(() => {
    if (open) {
      lastFocusedElement =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      void focusInitialControl();
      return;
    }

    if (lastFocusedElement) {
      lastFocusedElement.focus();
      lastFocusedElement = null;
    }
  });
</script>

{#if open}
  <div
    class="modal-shell"
    transition:fade={{ duration: 200 }}
  >
    <button
      type="button"
      class="modal-backdrop"
      aria-label="Close dialog"
      onclick={onClose}
    ></button>
    <div
      bind:this={dialogElement}
      class="modal-content"
      class:allow-overflow={allowOverflow}
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
      tabindex="-1"
      onkeydown={handleDialogKeydown}
      transition:fly={{ y: 20, duration: 300 }}
    >
      <div class="modal-header">
        <h3 id={titleId}>{title}</h3>
        <button
          bind:this={closeButtonElement}
          type="button"
          class="close-btn"
          aria-label="Close dialog"
          onclick={onClose}
        >
          ×
        </button>
      </div>
      <div class="modal-body" class:allow-overflow={allowOverflow}>
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-shell {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--modal-backdrop);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-backdrop {
    position: absolute;
    inset: 0;
    background-color: var(--modal-backdrop);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border: none;
    padding: 0;
  }

  .modal-content {
    position: relative;
    z-index: 1;
    background-color: var(--bg-primary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    max-width: 90%;
    width: 320px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
  }

  .modal-content.allow-overflow {
    overflow: visible;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md);
    border-bottom: 1px solid var(--border);
  }

  .modal-header h3 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    font-size: 24px;
    color: var(--text-secondary);
    transition: color var(--transition-fast);
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .modal-body {
    padding: var(--spacing-md);
    overflow-y: auto;
  }

  .modal-body.allow-overflow {
    overflow-y: visible;
  }
</style>
