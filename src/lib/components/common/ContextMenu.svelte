<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { fade } from "svelte/transition";

  interface ColorPickerItem {
    type: "colorPicker";
    label: string;
    currentColor: string;
    onSelect: (color: string) => void;
  }

  interface MenuItem {
    type?: "action" | "separator";
    label: string;
    icon?: string;
    onClick: () => void;
    danger?: boolean;
  }

  type AnyMenuItem = MenuItem | ColorPickerItem;

  let { items = [] }: { items: AnyMenuItem[] } = $props();

  let menuElement = $state<HTMLDivElement>();

  const PROJECT_COLORS = [
    { value: "#ef4444", label: "Red" },
    { value: "#f97316", label: "Orange" },
    { value: "#eab308", label: "Yellow" },
    { value: "#22c55e", label: "Green" },
    { value: "#3b82f6", label: "Blue" },
    { value: "#8b5cf6", label: "Violet" },
    { value: "#ec4899", label: "Pink" },
    { value: "#14b8a6", label: "Teal" },
  ];

  function handleOutsideClick(e: MouseEvent) {
    if (
      uiStore.contextMenuOpen &&
      menuElement &&
      !menuElement.contains(e.target as Node)
    ) {
      uiStore.closeContextMenu();
    }
  }

  // Adjust position to keep menu within viewport
  let adjustedPos = $derived.by(() => {
    if (!uiStore.contextMenuOpen) return { x: 0, y: 0 };

    const { x, y } = uiStore.contextMenuPos;
    const hasColorPicker = items.some(
      (i) => (i as ColorPickerItem).type === "colorPicker",
    );
    const menuWidth = 192;
    const menuHeight = items.length * 40 + (hasColorPicker ? 56 : 0);

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
      window.addEventListener("scroll", () => uiStore.closeContextMenu(), {
        passive: true,
      });
    } else {
      window.removeEventListener("mousedown", handleOutsideClick);
    }

    return () => {
      window.removeEventListener("mousedown", handleOutsideClick);
    };
  });

  function isColorPicker(item: AnyMenuItem): item is ColorPickerItem {
    return (item as ColorPickerItem).type === "colorPicker";
  }
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
      {#if isColorPicker(item)}
        <div class="color-picker-row">
          <span class="color-picker-label">{item.label}</span>
          <div class="color-swatches">
            {#each PROJECT_COLORS as color}
              <button
                class="color-swatch"
                class:active={item.currentColor === color.value}
                style="background-color: {color.value};"
                title={color.label}
                aria-label="Set project color to {color.label}"
                onclick={() => {
                  item.onSelect(color.value);
                  uiStore.closeContextMenu();
                }}
              ></button>
            {/each}
          </div>
        </div>
      {:else}
        <button
          class="menu-item"
          class:danger={(item as MenuItem).danger}
          onclick={() => {
            (item as MenuItem).onClick();
            uiStore.closeContextMenu();
          }}
          role="menuitem"
        >
          {#if (item as MenuItem).icon}
            <span class="menu-icon">{(item as MenuItem).icon}</span>
          {/if}
          <span class="menu-label">{item.label}</span>
        </button>
      {/if}
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
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    padding: 4px;
    min-width: 192px;
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

  /* Color Picker Row */
  .color-picker-row {
    padding: 6px var(--spacing-sm) 4px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    border-bottom: 1px solid var(--border-light);
    margin-bottom: 2px;
  }

  .color-picker-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 2px;
  }

  .color-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    padding: 0 2px;
  }

  .color-swatch {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition:
      transform 0.15s,
      border-color 0.15s,
      box-shadow 0.15s;
    padding: 0;
    outline: none;
  }

  .color-swatch:hover {
    transform: scale(1.25);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  }

  .color-swatch.active {
    border-color: var(--text-primary);
    transform: scale(1.1);
    box-shadow:
      0 0 0 2px var(--bg-primary),
      0 0 0 4px currentColor;
  }
</style>
