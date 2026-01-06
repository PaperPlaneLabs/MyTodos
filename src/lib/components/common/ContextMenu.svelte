<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { fade } from "svelte/transition";

  interface MenuItem {
    label: string;
    icon?: string;
    onClick: () => void;
    danger?: boolean;
  }

  let { items = [] }: { items: MenuItem[] } = $props();

  let menuElement = $state<HTMLDivElement>();

  function handleOutsideClick(e: MouseEvent) {
    if (uiStore.contextMenuOpen && menuElement && !menuElement.contains(e.target as Node)) {
      uiStore.closeContextMenu();
    }
  }

  // Adjust position to keep menu within viewport
  let adjustedPos = $derived.by(() => {
    if (!uiStore.contextMenuOpen) return { x: 0, y: 0 };
    
    const { x, y } = uiStore.contextMenuPos;
    // Simple adjustment, could be more robust with element measurements
    // but we'll assume a fixed max width/height for now or just use window width
    const menuWidth = 160;
    const menuHeight = items.length * 40;
    
    let adjX = x;
    let adjY = y;
    
    if (x + menuWidth > window.innerWidth) {
      adjX = x - menuWidth;
    }
    
    if (y + menuHeight > window.innerHeight) {
      adjY = y - menuHeight;
    }
    
    return { x: adjX, y: adjY };
  });

  $effect(() => {
    if (uiStore.contextMenuOpen) {
      window.addEventListener("mousedown", handleOutsideClick);
      // Also close on scroll
      window.addEventListener("scroll", () => uiStore.closeContextMenu(), { passive: true });
    } else {
      window.removeEventListener("mousedown", handleOutsideClick);
    }
    
    return () => {
      window.removeEventListener("mousedown", handleOutsideClick);
    };
  });
</script>

{#if uiStore.contextMenuOpen}
  <div
    bind:this={menuElement}
    class="context-menu"
    style="top: {adjustedPos.y}px; left: {adjustedPos.x}px;"
    transition:fade={{ duration: 100 }}
    role="menu"
    tabindex="-1"
  >
    {#each items as item}
      <button
        class="menu-item"
        class:danger={item.danger}
        onclick={() => {
          item.onClick();
          uiStore.closeContextMenu();
        }}
        role="menuitem"
      >
        {#if item.icon}
          <span class="menu-icon">{item.icon}</span>
        {/if}
        <span class="menu-label">{item.label}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 1000;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    padding: 4px;
    min-width: 160px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  :global([data-theme="dark"]) .context-menu {
    background-color: var(--bg-secondary);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border: none;
    background: none;
    width: 100%;
    text-align: left;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    transition: background-color var(--transition-fast);
  }

  .menu-item:hover {
    background-color: var(--bg-hover);
  }

  .menu-item.danger {
    color: var(--danger);
  }

  .menu-item.danger:hover {
    background-color: var(--danger-light);
  }

  .menu-icon {
    font-size: 14px;
    opacity: 0.7;
    width: 16px;
    display: flex;
    justify-content: center;
  }

  .menu-label {
    flex: 1;
  }
</style>
