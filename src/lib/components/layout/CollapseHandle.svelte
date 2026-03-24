<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { db } from "$lib/services/db";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { fly, fade } from "svelte/transition";

  let elapsed = $derived(Math.floor(timerStore.elapsed));
  let isDragging = $state(false);
  let hasMoved = $state(false);
  let startY = $state(0);
  let startScreenY = $state<number | null>(null);
  let startTop = $state(0);

  async function toggleCollapse(e: MouseEvent | PointerEvent) {
    // If we moved more than a tiny bit, it's a drag, not a click
    if (hasMoved) return;
    
    const newState = !uiStore.isCollapsed;
    uiStore.setCollapsed(newState);
    await db.window.setCollapsed(newState, uiStore.handleTop);
  }

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    isDragging = true;
    hasMoved = false;
    startScreenY = e.screenY;
    startY = e.clientY;
    startTop = uiStore.handleTop;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
  }

  async function handlePointerMove(e: PointerEvent) {
    if (!isDragging || startScreenY === null) return;
    
    const deltaY = e.screenY - startScreenY;
    
    // Only start moving if we've crossed a 3px threshold
    if (!hasMoved) {
      if (Math.abs(deltaY) < 3) return;
      hasMoved = true;
    }
    
    let newTop = startTop + deltaY;
    
    const padding = 10;
    const handleHeight = 140;
    
    try {
      const { currentMonitor } = await import("@tauri-apps/api/window");
      const monitor = await currentMonitor();
      if (monitor) {
        const scaleFactor = monitor.scaleFactor;
        const workArea = monitor.workArea;
        const monitorTop = workArea.position.y / scaleFactor;
        const monitorHeight = workArea.size.height / scaleFactor;
        const monitorWidth = workArea.size.width / scaleFactor;
        const monitorLeft = workArea.position.x / scaleFactor;
        
        const maxTop = monitorHeight - (uiStore.isCollapsed ? handleHeight : 56) - padding;
        newTop = Math.max(padding, Math.min(newTop, maxTop));
        
        uiStore.setHandleTop(newTop);

        if (uiStore.isCollapsed) {
          const logicalX = monitorLeft + monitorWidth - 44;
          const logicalY = monitorTop + newTop;
          await db.window.move(logicalX, logicalY);
        }
      }
    } catch (err) {
      console.error("Drag error:", err);
      uiStore.setHandleTop(newTop);
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isDragging) return;
    isDragging = false;
    startScreenY = null;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  }
</script>

<div 
  class="collapse-handle-wrapper" 
  class:collapsed={uiStore.isCollapsed}
  style={uiStore.isCollapsed ? "top: 0" : `top: ${uiStore.handleTop}px`}
>
  {#if uiStore.isCollapsed}
    <button 
      type="button"
      class="handle" 
      aria-label={uiStore.isCollapsed ? "Expand the app" : "Collapse the app"}
      onclick={toggleCollapse}
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      transition:fade={{ duration: 200 }}
      class:dragging={isDragging}
    >
      <div class="handle-content">
        <div class="timer-vertical">
          {#if timerStore.active}
            <div class="timer-text" class:running={timerStore.isRunning}>
              <TimeDisplay seconds={elapsed} format="short" />
            </div>
            <div class="timer-dot" class:running={timerStore.isRunning}></div>
          {:else}
            <span class="idle-icon">⏱</span>
          {/if}
        </div>
        <div class="expand-icon">«</div>
      </div>
    </button>
  {:else}
    <button 
      type="button"
      class="collapse-btn" 
      aria-label="Collapse the app"
      onclick={toggleCollapse}
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      title="Collapse to right"
      class:dragging={isDragging}
    >
      »
    </button>
  {/if}
</div>

<style>
  .collapse-handle-wrapper {
    position: fixed;
    right: 0;
    z-index: 10000;
    touch-action: none;
    width: 44px; /* Fix wrapper width to handle size */
    display: flex;
    justify-content: flex-end;
  }

  .collapse-handle-wrapper.collapsed {
    left: 0;
    width: 100%; /* Take full width of the now-small window */
  }

  .handle {
    width: 100%;
    height: 140px;
    background: var(--accent);
    color: var(--accent-contrast);
    border: none;
    border-radius: 0; /* Square edges when collapsed against screen edge */
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    box-shadow: none; /* Shadow not needed when window is tiny */
    padding: 0;
    transition: background var(--transition-normal);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-left: none;
  }

  .handle.dragging {
    background: var(--accent-hover);
    box-shadow: 6px 0 20px rgba(0, 0, 0, 0.4);
  }

  .handle:hover {
    background: var(--accent-hover);
    width: 48px;
  }

  .handle:active {
    cursor: grabbing;
  }

  .handle-content {
    pointer-events: none; /* Let the button handle the dragging */
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    height: 100%;
    padding: var(--spacing-sm) 0;
  }

  .timer-vertical {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    flex: 1;
    justify-content: center;
  }

  .timer-text {
    font-family: var(--font-mono);
    font-size: 15px;
    font-weight: 800;
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
    letter-spacing: 1px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .timer-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.5);
  }

  .timer-dot.running {
    background: #4ade80;
    box-shadow: 0 0 10px #4ade80, 0 0 20px rgba(74, 222, 128, 0.5);
    animation: blink 1.5s ease-in-out infinite;
  }

  .expand-icon {
    font-size: 24px;
    font-weight: bold;
    opacity: 0.8;
  }

  .collapse-btn {
    width: 28px;
    height: 56px;
    background: var(--accent);
    color: var(--accent-contrast);
    border: none;
    border-radius: var(--radius-md) 0 0 var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    transition: width var(--transition-fast), background var(--transition-fast);
    box-shadow: var(--shadow-md);
    z-index: 1000;
    margin-right: -16px; /* Shift it slightly left so it's more visible on the edge */
  }

  .collapse-btn:hover {
    background: var(--accent-hover);
    width: 32px;
    margin-right: -12px;
  }

  .idle-icon {
    font-size: 20px;
    opacity: 0.7;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
