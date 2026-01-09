<script lang="ts">
  import { uiStore } from "$lib/stores/ui.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { db } from "$lib/services/db";
  import TimeDisplay from "$lib/components/common/TimeDisplay.svelte";
  import { fly, fade } from "svelte/transition";

  let elapsed = $derived(Math.floor(timerStore.elapsed));
  let isDragging = $state(false);
  let startY = $state(0);
  let startTop = $state(0);

  async function toggleCollapse(e: MouseEvent | PointerEvent) {
    // Prevent toggle if we were dragging
    if (Math.abs(startY - (e as PointerEvent).clientY) > 5) return;
    
    const newState = !uiStore.isCollapsed;
    uiStore.setCollapsed(newState);
    await db.window.setCollapsed(newState);
  }

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // Only left click
    isDragging = true;
    startY = e.clientY;
    startTop = uiStore.handleTop;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isDragging) return;
    
    const deltaY = e.clientY - startY;
    let newTop = startTop + deltaY;
    
    // Boundary checks (prevent dragging off screen)
    const padding = 10;
    const handleHeight = uiStore.isCollapsed ? 140 : 56;
    const maxTop = window.innerHeight - handleHeight - padding;
    
    newTop = Math.max(padding, Math.min(newTop, maxTop));
    uiStore.setHandleTop(newTop);
  }

  function handlePointerUp(e: PointerEvent) {
    if (!isDragging) return;
    isDragging = false;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  }
</script>

<div 
  class="collapse-handle-wrapper" 
  class:collapsed={uiStore.isCollapsed}
  style="top: {uiStore.handleTop}px"
>
  {#if uiStore.isCollapsed}
    <button 
      class="handle" 
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
      class="collapse-btn" 
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
  }

  .collapse-handle-wrapper.collapsed {
    left: 0;
    right: auto;
    width: 44px;
  }

  .handle {
    width: 44px;
    height: 140px;
    background: var(--accent);
    color: white;
    border: none;
    border-radius: 0 var(--radius-lg) var(--radius-lg) 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    box-shadow: 4px 0 15px rgba(0, 0, 0, 0.3);
    padding: 0;
    transition: width var(--transition-normal), background var(--transition-normal);
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
    color: white;
    border: none;
    border-radius: var(--radius-md) 0 0 var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ns-resize;
    transition: width var(--transition-fast), background var(--transition-fast);
    box-shadow: var(--shadow-md);
    z-index: 1000;
  }

  .collapse-btn.dragging {
    background: var(--accent-hover);
    box-shadow: var(--shadow-lg);
  }

  .collapse-btn:hover {
    background: var(--accent-hover);
    width: 32px;
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
