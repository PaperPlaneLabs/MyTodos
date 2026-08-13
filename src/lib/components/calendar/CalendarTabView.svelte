<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import CalendarHeader from "$lib/components/calendar/CalendarHeader.svelte";
  import CalendarInspector from "$lib/components/calendar/CalendarInspector.svelte";
  import CalendarMonth from "$lib/components/calendar/CalendarMonth.svelte";
  import CalendarSkeleton from "$lib/components/calendar/CalendarSkeleton.svelte";
  import CalendarWeek from "$lib/components/calendar/CalendarWeek.svelte";
  import { db } from "$lib/services/db";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";

  let isInitializing = $state(true);
  let isPortrait = $derived(uiStore.windowOrientation === "left" || uiStore.windowOrientation === "right");

  onMount(async () => {
    calendarStore.initPreferences();
    try {
      await calendarStore.ensureCurrentRangeLoaded();
      const orientation = await db.window.getOrientation();
      if (orientation.side === "left" || orientation.side === "right" || orientation.side === "center") {
        uiStore.setWindowOrientation(orientation.side);
      }
    } catch (error) {
      console.error("Failed to initialize calendar:", error);
    } finally {
      isInitializing = false;
    }
  });
</script>

<section class="calendar-page" class:portrait={isPortrait} transition:fade={{ duration: 140 }}>
  <CalendarHeader />
  <div class="calendar-workspace" class:has-inspector={calendarStore.inspectorOpen || !!calendarStore.editorDraft}>
    <main class="calendar-canvas">
      {#if isInitializing && calendarStore.isLoading}
        <CalendarSkeleton />
      {:else if calendarStore.error && calendarStore.allItems.length === 0}
        <div class="calendar-error" role="alert">
          <h3>Calendar could not load</h3>
          <p>{calendarStore.error}</p>
          <button type="button" onclick={() => calendarStore.refreshCurrentRange()}>Try again</button>
        </div>
      {:else}
        {#if calendarStore.error}
          <div class="retained-error" role="status">Some calendar data could not refresh. Showing the last loaded range.</div>
        {/if}
        <div class="view-stage">
          {#key calendarStore.viewMode}
            <div class="view-transition" in:fade={{ duration: 150 }}>
              {#if calendarStore.viewMode === "month"}<CalendarMonth />{:else}<CalendarWeek />{/if}
            </div>
          {/key}
        </div>
      {/if}
    </main>

    {#if calendarStore.inspectorOpen || calendarStore.editorDraft}
      {#if isPortrait}<button type="button" class="sheet-backdrop" aria-label="Close calendar details" onclick={() => { calendarStore.closeEventEditor(); calendarStore.closeInspector(); }}></button>{/if}
      <div class="inspector-panel" transition:fade={{ duration: 120 }}><CalendarInspector /></div>
    {/if}
  </div>
</section>

<style>
  .calendar-page { height:100%; min-height:0; display:flex; flex-direction:column; overflow:hidden; background:var(--bg-primary); }
  .calendar-workspace { flex:1; min-height:0; display:grid; grid-template-columns:minmax(0,1fr); position:relative; }
  .calendar-workspace.has-inspector { grid-template-columns:minmax(0,1fr) minmax(280px,330px); }
  .calendar-canvas { min-width:0; min-height:0; display:flex; flex-direction:column; overflow:hidden; }
  .view-stage, .view-transition { min-height:0; flex:1; display:flex; flex-direction:column; }
  .inspector-panel { min-width:0; min-height:0; border-left:1px solid var(--border); overflow:hidden; box-shadow:-6px 0 18px rgba(0,0,0,.05); z-index:20; }
  .retained-error { flex:0 0 auto; padding:5px 10px; background:var(--warning-light,var(--bg-hover)); color:var(--warning); font-size:10px; text-align:center; }
  .calendar-error { margin:auto; max-width:360px; padding:var(--spacing-xl); text-align:center; color:var(--text-secondary); }
  .calendar-error h3 { color:var(--text-primary); }
  .calendar-error button { border:1px solid var(--accent); border-radius:var(--radius-md); background:var(--accent); color:var(--accent-contrast); padding:7px 12px; cursor:pointer; }
  .portrait .calendar-workspace.has-inspector { grid-template-columns:minmax(0,1fr); }
  .portrait .inspector-panel { position:absolute; left:0; right:0; bottom:0; height:min(68%,560px); border-left:0; border-top:1px solid var(--border); border-radius:var(--radius-lg) var(--radius-lg) 0 0; box-shadow:0 -12px 36px rgba(0,0,0,.24); }
  .sheet-backdrop { position:absolute; inset:0; border:0; background:rgba(0,0,0,.18); z-index:19; }
  @media (max-width:900px) and (min-width:601px) { .calendar-workspace.has-inspector { grid-template-columns:minmax(0,1fr) 290px; } }
  @media (max-width:600px) { .calendar-workspace.has-inspector { grid-template-columns:minmax(0,1fr); } .inspector-panel { position:absolute; left:0; right:0; bottom:0; height:min(72%,600px); border-left:0; border-top:1px solid var(--border); z-index:20; } }
  @media (prefers-reduced-motion:reduce) { :global(.view-transition) { transition:none !important; } }
</style>
