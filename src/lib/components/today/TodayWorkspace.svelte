<script lang="ts">
  import { onMount } from "svelte";

  import { googleCalendarStore } from "$lib/stores/google-calendar.svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { timerStore } from "$lib/stores/timer.svelte";
  import { todayStore } from "$lib/stores/today.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";

  import TodayActiveTimerCard from "./TodayActiveTimerCard.svelte";
  import TodayEventRow from "./TodayEventRow.svelte";
  import TodayProgressCard from "./TodayProgressCard.svelte";
  import TodayTaskRow from "./TodayTaskRow.svelte";
  import { scheduleLocalMidnightRefresh } from "./today-refresh";
  import { buildTodayAgenda } from "./today-view-utils";

  let {
    onCompleteTask,
    onToggleTimer,
    onTaskContextMenu,
  }: {
    onCompleteTask: (taskId: number) => void | Promise<void>;
    onToggleTimer: (taskId: number) => void | Promise<void>;
    onTaskContextMenu: (event: MouseEvent, taskId: number) => void;
  } = $props();

  let mounted = $state(false);
  let wasTaskModalOpen = false;

  const isPortrait = $derived(
    uiStore.windowOrientation === "left" || uiStore.windowOrientation === "right",
  );
  const agenda = $derived(buildTodayAgenda(todayStore.taskSummary.today, todayStore.events));
  const remainingCount = $derived(
    todayStore.taskSummary.overdue.length + todayStore.taskSummary.today.length,
  );
  const headingDate = $derived(
    todayStore.date
      ? new Date(`${todayStore.date}T12:00:00`).toLocaleDateString("en-US", {
          weekday: "long",
          month: "long",
          day: "numeric",
        })
      : "Your day",
  );

  onMount(() => {
    mounted = true;
    void todayStore.refresh();
    const refreshOnFocus = () => void todayStore.refresh();
    const cancelMidnightRefresh = scheduleLocalMidnightRefresh(refreshOnFocus);
    window.addEventListener("focus", refreshOnFocus);

    return () => {
      mounted = false;
      cancelMidnightRefresh();
      window.removeEventListener("focus", refreshOnFocus);
    };
  });

  $effect(() => {
    const modalOpen = uiStore.showTaskModal;
    if (mounted && wasTaskModalOpen && !modalOpen) {
      void todayStore.refresh();
    }
    wasTaskModalOpen = modalOpen;
  });

  $effect(() => {
    timerStore.changeSignal;
    if (mounted) void todayStore.refresh();
  });

  function editTask(taskId: number) {
    uiStore.openTaskModal({ taskId });
  }

  function openEventInCalendar() {
    calendarStore.setSelectedDate(new Date(`${todayStore.date}T12:00:00`));
    uiStore.openCalendarView();
  }
</script>

<main
  class="today-workspace"
  class:portrait={isPortrait}
  aria-labelledby="today-heading"
>
  <header class="today-header">
    <div>
      <p class="eyebrow">{headingDate}</p>
      <h2 id="today-heading">Today</h2>
      {#if todayStore.date}
        <p class="day-summary">
          {remainingCount === 0 ? "Your day is clear" : `${remainingCount} task${remainingCount === 1 ? "" : "s"} remaining`}
        </p>
      {/if}
    </div>
    <button
      type="button"
      class="refresh-btn"
      aria-label="Refresh Today workspace"
      disabled={todayStore.loading}
      onclick={() => todayStore.refresh()}
    >
      <span aria-hidden="true">↻</span>
      {todayStore.loading ? "Refreshing" : "Refresh"}
    </button>
  </header>

  <TodayActiveTimerCard />

  {#if todayStore.loading && !todayStore.date}
    <div class="workspace-column" aria-live="polite" aria-busy="true">
      <div class="skeleton skeleton-wide"></div>
      <div class="skeleton"></div>
      <div class="skeleton"></div>
    </div>
  {:else}
    {#if todayStore.error}
      <div class="state-card error" role="alert">
        <span>{todayStore.error}. Showing the last available snapshot.</span>
        <button type="button" onclick={() => todayStore.refresh()}>Try again</button>
      </div>
    {/if}

    <div class="workspace-column">
      <div class="work-column">
        {#if todayStore.taskSummary.overdue.length > 0}
          <section class="today-section overdue-section" aria-labelledby="overdue-heading">
            <div class="section-heading">
              <div>
                <p class="section-kicker danger">Needs attention</p>
                <h3 id="overdue-heading">Overdue</h3>
              </div>
              <span class="count danger-count">{todayStore.taskSummary.overdue.length}</span>
            </div>
            <div class="item-list">
              {#each todayStore.taskSummary.overdue as task (task.id)}
                <TodayTaskRow
                  {task}
                  overdue
                  onEdit={editTask}
                  onComplete={onCompleteTask}
                  {onToggleTimer}
                  onContextMenu={onTaskContextMenu}
                />
              {/each}
            </div>
          </section>
        {/if}

        <section class="today-section agenda-section" aria-labelledby="agenda-heading">
          <div class="section-heading">
            <div>
              <p class="section-kicker">Up next</p>
              <h3 id="agenda-heading">Agenda</h3>
            </div>
            <span class="agenda-meta">{todayStore.taskSummary.today.length} task{todayStore.taskSummary.today.length === 1 ? "" : "s"} · {todayStore.events.length} event{todayStore.events.length === 1 ? "" : "s"}</span>
          </div>
          {#if !googleCalendarStore.connected}<p class="calendar-notice">Calendar not connected. Tasks remain available here.</p>{/if}
          {#if todayStore.taskSummary.today.length === 0 && todayStore.events.length === 0}
            <div class="empty-state">
              <span aria-hidden="true">✓</span>
              <div><strong>Your agenda is clear</strong><small>Add a dated task when you are ready.</small></div>
              <button type="button" onclick={() => uiStore.openTaskModal({ deadline: todayStore.date })}>Add task</button>
            </div>
          {:else}
            <div class="agenda-groups">
              {#if agenda.allDay.length}<div class="agenda-group"><p class="agenda-label">All day</p><div class="item-list">
                {#each agenda.allDay as item (`event-${item.id}`)}<TodayEventRow event={item.event} onOpen={openEventInCalendar} />{/each}
              </div></div>{/if}
              {#if agenda.anytime.length}<div class="agenda-group"><p class="agenda-label">Anytime</p><div class="item-list">
                {#each agenda.anytime as item (`task-${item.id}`)}<TodayTaskRow task={item.task} onEdit={editTask} onComplete={onCompleteTask} {onToggleTimer} onContextMenu={onTaskContextMenu} />{/each}
              </div></div>{/if}
              {#if agenda.timeline.length}<div class="agenda-group"><p class="agenda-label">Timeline</p><div class="item-list">
                {#each agenda.timeline as item (`${item.kind}-${item.id}`)}
                  {#if item.kind === "event"}<TodayEventRow event={item.event} onOpen={openEventInCalendar} />{:else}<TodayTaskRow task={item.task} onEdit={editTask} onComplete={onCompleteTask} {onToggleTimer} onContextMenu={onTaskContextMenu} />{/if}
                {/each}
              </div></div>{/if}
            </div>
          {/if}
        </section>
      </div>
      <TodayProgressCard completed={todayStore.taskSummary.completed_today} total={todayStore.taskSummary.total_today} />
    </div>
  {/if}
</main>

<style>
  .today-workspace { flex: none; min-height: max-content; overflow: visible; padding: clamp(var(--spacing-md), 2.5vw, 28px); display: flex; flex-direction: column; gap: var(--spacing-lg); }
  .today-header, .section-heading, .state-card { display: flex; align-items: center; justify-content: space-between; gap: var(--spacing-md); }
  .eyebrow, .section-kicker, h2, h3, .day-summary { margin: 0; }
  .eyebrow, .section-kicker { color: var(--text-tertiary); font-size: var(--text-xs); font-weight: 700; text-transform: uppercase; letter-spacing: .07em; }
  .section-kicker.danger { color: var(--danger); }
  h2 { margin-top: 3px; font-size: 28px; line-height: 1.08; letter-spacing: -.025em; }
  h3 { margin-top: 2px; font-size: var(--text-base); }
  .day-summary { margin-top: var(--spacing-xs); color: var(--text-secondary); font-size: var(--text-sm); }
  .refresh-btn, .state-card button, .empty-state button { display: inline-flex; align-items: center; gap: var(--spacing-xs); border: 1px solid transparent; background: transparent; color: var(--text-secondary); border-radius: var(--radius-md); padding: var(--spacing-xs) var(--spacing-sm); cursor: pointer; }
  .refresh-btn:hover, .state-card button:hover, .empty-state button:hover { color: var(--text-primary); background: var(--bg-hover); }
  .refresh-btn:disabled { opacity: .6; cursor: wait; }
  .workspace-column, .work-column { display: flex; min-width: 0; flex-direction: column; gap: var(--spacing-lg); }
  .today-section, .state-card { padding: var(--spacing-md); border: 1px solid var(--border-light); border-radius: var(--radius-lg); background: var(--bg-secondary); }
  .today-section { display: flex; flex-direction: column; gap: var(--spacing-md); box-shadow: 0 1px 2px color-mix(in srgb, var(--text-primary) 4%, transparent); }
  .overdue-section { position: relative; border-color: var(--border-light); }
  .overdue-section::before { content: ""; position: absolute; inset: var(--spacing-md) auto var(--spacing-md) 0; width: 2px; border-radius: 2px; background: color-mix(in srgb, var(--danger) 65%, transparent); }
  .state-card.error { color: var(--danger); background: var(--danger-light); }
  .count { min-width: 20px; color: var(--text-tertiary); font-size: var(--text-xs); font-weight: 700; text-align: right; }
  .danger-count { color: var(--danger); }
  .item-list { display: flex; flex-direction: column; gap: 2px; }
  small { color: var(--text-tertiary); font-size: var(--text-xs); }
  .agenda-meta, .calendar-notice { color: var(--text-tertiary); font-size: var(--text-xs); }
  .calendar-notice { margin: calc(-1 * var(--spacing-xs)) 0 0; }
  .agenda-groups, .agenda-group { display: flex; flex-direction: column; gap: var(--spacing-sm); }
  .agenda-label { margin: 0; color: var(--text-tertiary); font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; }
  .empty-state { min-height: 90px; display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-md); border-radius: var(--radius-md); color: var(--text-secondary); background: color-mix(in srgb, var(--bg-primary) 55%, transparent); }
  .empty-state > span { display: grid; width: 30px; height: 30px; place-items: center; border-radius: 50%; color: var(--success); background: var(--success-light); font-weight: 800; }
  .empty-state > div { flex: 1; display: flex; flex-direction: column; }
  .skeleton { min-height: 180px; border-radius: var(--radius-lg); background: linear-gradient(90deg, var(--bg-secondary), var(--bg-hover), var(--bg-secondary)); background-size: 200% 100%; animation: shimmer 1.4s infinite; }
  .skeleton-wide { grid-column: 1 / -1; min-height: 110px; }
  .portrait { padding: var(--spacing-md); gap: var(--spacing-md); }
  .portrait .workspace-column, .portrait .work-column { gap: var(--spacing-md); }
  :global(body.compact-mode) .today-workspace { padding: var(--spacing-sm); gap: var(--spacing-sm); }
  :global(body.compact-mode) .today-section { padding: var(--spacing-sm); gap: var(--spacing-sm); }
  @media (prefers-reduced-motion: reduce) { .skeleton { animation: none; transition: none; } }
  @keyframes shimmer { from { background-position: 100% 0; } to { background-position: -100% 0; } }
</style>
