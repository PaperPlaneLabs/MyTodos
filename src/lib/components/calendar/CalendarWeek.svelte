<script lang="ts">
  import { onMount, tick } from "svelte";
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { dateToKey } from "$lib/components/calendar/calendar-utils";
  import type { CalendarDay, CalendarItem } from "$lib/types/calendar";

  let feedElement = $state<HTMLDivElement | null>(null);
  let isProgrammaticScrolling = false;
  let isLoadingMore = $state(false);
  let dragOverDateKey = $state<string | null>(null);

  // Multi-week data from store
  let weekGroups = $derived(calendarStore.generateMultiWeekGroups());
  let currentFocusDate = $derived(calendarStore.currentDate);
  let currentWeekDays = $derived(calendarStore.generateWeekDays(currentFocusDate));
  let todayKey = $derived(dateToKey(new Date()));

  // Active day currently visible near the top
  let activeDayKey = $state(dateToKey(new Date()));

  onMount(() => {
    void tick().then(() => {
      const initialDate = calendarStore.selectedDate ?? calendarStore.currentDate ?? new Date();
      scrollToDay(dateToKey(initialDate), "instant");
    });
  });

  // Watch for external scroll requests from CalendarHeader (<, >, Today buttons or jump dialog)
  $effect(() => {
    const target = calendarStore.scrollTarget;
    if (target && feedElement) {
      scrollToDay(dateToKey(target.date), target.behavior);
      calendarStore.clearScrollTarget();
    }
  });

  function formatWeekday(date: Date): string {
    return date.toLocaleDateString("en-US", { weekday: "short" }).toUpperCase();
  }

  function formatMonthHeader(date: Date): string {
    return date.toLocaleDateString("en-US", { month: "long", year: "numeric" });
  }

  function shouldShowMonthHeader(day: CalendarDay, dayIndex: number, weekIndex: number): boolean {
    if (weekIndex === 0 && dayIndex === 0) return true;
    return day.date.getDate() === 1;
  }

  function formatCardTime(item: CalendarItem): string {
    if (item.isAllDay) return "All day";
    const format = (timeStr: string | null) => {
      if (!timeStr) return "";
      const [hours, minutes] = timeStr.split(":").map(Number);
      const period = hours >= 12 ? "PM" : "AM";
      const displayHours = hours % 12 || 12;
      return `${displayHours}:${String(minutes).padStart(2, "0")} ${period}`;
    };
    if (item.startTime && item.endTime) {
      return `${format(item.startTime)} – ${format(item.endTime)}`;
    }
    return format(item.startTime);
  }

  function getProject(projectId: number | null | undefined) {
    if (!projectId) return null;
    return projectStore.projects.find((p) => p.id === projectId) ?? null;
  }

  async function toggleTask(event: MouseEvent, taskId: number) {
    event.stopPropagation();
    await calendarStore.toggleTask(taskId);
  }

  function selectCard(event: MouseEvent, item: CalendarItem) {
    event.stopPropagation();
    calendarStore.selectItem(item);
  }

  function scrollToDay(dateKey: string, behavior: ScrollBehavior = "smooth") {
    if (!feedElement) return;
    const targetEl = feedElement.querySelector<HTMLElement>(`[data-date-key="${dateKey}"]`);
    if (targetEl) {
      isProgrammaticScrolling = true;
      targetEl.scrollIntoView({ behavior, block: "start" });
      activeDayKey = dateKey;
      setTimeout(() => {
        isProgrammaticScrolling = false;
      }, 500);
    } else {
      const [year, month, day] = dateKey.split("-").map(Number);
      calendarStore.setCurrentDate(new Date(year, month - 1, day), true);
    }
  }

  async function handleScroll() {
    if (!feedElement || isProgrammaticScrolling || isLoadingMore) return;
    const { scrollTop, scrollHeight, clientHeight } = feedElement;

    // Infinite scroll: Near bottom -> load future weeks
    if (scrollHeight - scrollTop - clientHeight < 350) {
      isLoadingMore = true;
      await calendarStore.loadMoreFutureWeeks(4);
      isLoadingMore = false;
    }

    // Infinite scroll: Near top -> load past weeks
    if (scrollTop < 120) {
      isLoadingMore = true;
      const prevHeight = feedElement.scrollHeight;
      await calendarStore.loadMorePastWeeks(2);
      await tick();
      const newHeight = feedElement.scrollHeight;
      feedElement.scrollTop = scrollTop + (newHeight - prevHeight);
      isLoadingMore = false;
    }

    // Sync visible day with header
    updateActiveVisibleDay();
  }

  function updateActiveVisibleDay() {
    if (!feedElement) return;
    const feedRect = feedElement.getBoundingClientRect();
    const dayElements = feedElement.querySelectorAll<HTMLElement>(".day-group[data-date-key]");

    for (const dayEl of dayElements) {
      const rect = dayEl.getBoundingClientRect();
      if (rect.bottom > feedRect.top + 36) {
        const dateKey = dayEl.dataset.dateKey;
        if (dateKey && dateKey !== activeDayKey) {
          activeDayKey = dateKey;
          const [y, m, d] = dateKey.split("-").map(Number);
          const visibleDate = new Date(y, m - 1, d);
          calendarStore.setCurrentDateSilent(visibleDate);
        }
        break;
      }
    }
  }

  function dragStart(event: DragEvent, item: CalendarItem) {
    if (item.readOnly || item.kind === "time_entry" || (item.kind === "local_event" && !!item.event.recurrence_rule)) return;
    event.dataTransfer?.setData("calendar-item-key", item.key);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
  }

  async function dropOnDay(event: DragEvent, dateKey: string) {
    event.preventDefault();
    dragOverDateKey = null;
    const key = event.dataTransfer?.getData("calendar-item-key");
    const item = calendarStore.allItems.find((candidate) => candidate.key === key);
    if (item) {
      await calendarStore.rescheduleItem(item, dateKey, item.startTime ?? null);
    }
  }
</script>

<div class="calendar-widget-week">
  <!-- Sticky Week Strip (Google Calendar widget top day-selector) -->
  <div class="sticky-week-strip" aria-label="Days in week">
    {#each currentWeekDays as day}
      {@const key = dateToKey(day.date)}
      {@const isToday = key === todayKey}
      {@const isActive = key === activeDayKey}
      {@const dayItems = calendarStore.getItemsForDate(key)}
      <button
        type="button"
        class="strip-day-btn"
        class:today={isToday}
        class:active={isActive}
        onclick={() => scrollToDay(key, "smooth")}
        aria-label={`${day.dayName} ${day.date.getDate()}`}
      >
        <span class="strip-day-name">{day.dayName}</span>
        <span class="strip-day-num">{day.date.getDate()}</span>
        {#if dayItems.length > 0}
          <span class="strip-item-dot" style={`--dot-color: ${dayItems[0].color}`}></span>
        {:else}
          <span class="strip-item-dot empty"></span>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Vertically Scrollable Continuous Multi-Week Card Feed -->
  <div
    class="week-feed-scroll"
    bind:this={feedElement}
    onscroll={handleScroll}
    role="feed"
    aria-label="Weekly schedule feed"
  >
    {#if isLoadingMore}
      <div class="feed-loader top" role="status"><span>Loading previous weeks...</span></div>
    {/if}

    {#each weekGroups as week, weekIdx (week.weekKey)}
      <div class="week-section" data-week-key={week.weekKey}>
        {#each week.days as day, dayIdx (day.dateKey)}
          {@const isMonthStart = shouldShowMonthHeader(day, dayIdx, weekIdx)}

          <!-- Month Transition Header Banner -->
          {#if isMonthStart}
            <div class="month-divider">
              <span class="month-title">{formatMonthHeader(day.date)}</span>
            </div>
          {/if}

          <!-- Day Group Block -->
          <div
            class="day-group"
            class:today={day.isToday}
            class:selected={calendarStore.selectedDate && dateToKey(calendarStore.selectedDate) === day.dateKey}
            class:drag-over={dragOverDateKey === day.dateKey}
            data-date-key={day.dateKey}
            data-week-key={week.weekKey}
            role="region"
            aria-label={`Schedule for ${day.dateKey}`}
            ondragover={(e) => { e.preventDefault(); dragOverDateKey = day.dateKey; }}
            ondragleave={(e) => {
              if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) {
                dragOverDateKey = null;
              }
            }}
            ondrop={(e) => dropOnDay(e, day.dateKey)}
          >
            <!-- Left Date Badge / Gutter -->
            <div class="day-badge-col">
              <div class="date-badge-box" class:today={day.isToday}>
                <span class="badge-weekday">{formatWeekday(day.date)}</span>
                <span class="badge-daynum">{day.date.getDate()}</span>
                {#if day.isToday}
                  <span class="badge-today-pill">TODAY</span>
                {/if}
              </div>
              <button
                type="button"
                class="day-add-btn"
                title={`Add task or event on ${day.dateKey}`}
                aria-label={`Add item on ${day.dateKey}`}
                onclick={() => calendarStore.openNewEvent(day.dateKey, null)}
              >
                +
              </button>
            </div>

            <!-- Right: Task & Event Cards Stream -->
            <div class="day-cards-stream">
              {#if day.items.length === 0}
                <div class="empty-day-card">
                  <span class="empty-text">No tasks or events</span>
                  <button
                    type="button"
                    class="empty-add-btn"
                    onclick={() => calendarStore.openNewEvent(day.dateKey, null)}
                  >
                    + Add
                  </button>
                </div>
              {:else}
                {#each day.items as item (item.key)}
                  <!-- Google Calendar Widget Style Card -->
                  <div
                    class={`agenda-card ${item.source}`}
                    class:completed={item.kind === "task" && item.task.completed}
                    class:selected={calendarStore.selectedItemKey === item.key}
                    style={`--item-color: ${item.color}`}
                    draggable={!item.readOnly && item.kind !== "time_entry" && !(item.kind === "local_event" && !!item.event.recurrence_rule)}
                    ondragstart={(e) => dragStart(e, item)}
                    onclick={(e) => selectCard(e, item)}
                    role="button"
                    tabindex="0"
                    onkeydown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        calendarStore.selectItem(item);
                      }
                    }}
                  >
                    <!-- Left Colored Accent Stripe -->
                    <div class="card-accent-bar" style={`background: ${item.color}`}></div>

                    <div class="card-body">
                      <!-- Top Line: Checkbox / Title / Badges -->
                      <div class="card-top-row">
                        {#if item.kind === "task"}
                          <button
                            type="button"
                            class="task-checkbox"
                            class:checked={item.task.completed}
                            aria-label={item.task.completed ? "Mark active" : "Mark complete"}
                            onclick={(e) => toggleTask(e, item.task.id)}
                          >
                            {#if item.task.completed}
                              <svg viewBox="0 0 16 16" width="11" height="11" fill="currentColor">
                                <path d="M13.854 3.646a.5.5 0 0 1 0 .708l-7 7a.5.5 0 0 1-.708 0l-3.5-3.5a.5.5 0 1 1 .708-.708L6.5 10.293l6.646-6.647a.5.5 0 0 1 .708 0z"/>
                              </svg>
                            {/if}
                          </button>
                        {/if}

                        <span class="card-title" title={item.title}>{item.title}</span>

                        <!-- Source / Project Badge -->
                        {#if item.kind === "task"}
                          {@const proj = getProject(item.task.project_id)}
                          {#if proj}
                            <span class="badge-pill project-pill" style={`--proj-color: ${proj.color || item.color}`}>
                              <span class="proj-dot"></span>
                              {proj.name}
                            </span>
                          {/if}
                        {:else if item.kind === "google_event"}
                          <span class="badge-pill google-pill">Google</span>
                        {:else if item.kind === "local_event" && item.event.recurrence_rule}
                          <span class="badge-pill repeat-pill" title="Recurring event">↻</span>
                        {:else if item.kind === "time_entry"}
                          <span class="badge-pill time-pill">⏱ Actual</span>
                        {/if}
                      </div>

                      <!-- Sub Line: Time / Duration / Description -->
                      <div class="card-meta-row">
                        {#if item.isAllDay}
                          <span class="meta-tag all-day">All day</span>
                        {:else if item.startTime}
                          <span class="meta-tag time">
                            <svg viewBox="0 0 16 16" width="10" height="10" fill="currentColor">
                              <path d="M8 3.5a.5.5 0 0 0-1 0V9a.5.5 0 0 0 .252.434l3.5 2a.5.5 0 0 0 .496-.868L8 8.71V3.5z"/>
                              <path d="M8 16A8 8 0 1 0 8 0a8 8 0 0 0 0 16zm7-8A7 7 0 1 1 1 8a7 7 0 0 1 14 0z"/>
                            </svg>
                            {formatCardTime(item)}
                          </span>
                        {/if}

                        {#if item.kind === "task" && item.task.planned_duration_minutes}
                          <span class="meta-tag duration">{item.task.planned_duration_minutes}m</span>
                        {/if}

                        {#if item.kind === "time_entry"}
                          <span class="meta-tag duration">{Math.round(item.entry.duration_seconds / 60)}m logged</span>
                        {/if}

                        {#if item.description}
                          <span class="meta-desc" title={item.description}>{item.description}</span>
                        {/if}
                      </div>
                    </div>
                  </div>
                {/each}
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/each}

    {#if isLoadingMore}
      <div class="feed-loader bottom" role="status"><span>Loading next weeks...</span></div>
    {/if}
  </div>
</div>

<style>
  .calendar-widget-week {
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    overflow: hidden;
  }

  /* ── Sticky Top Week Strip ── */
  .sticky-week-strip {
    flex: 0 0 auto;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 3px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    z-index: 10;
  }

  .strip-day-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 4px 2px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background 0.15s, color 0.15s;
  }

  .strip-day-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .strip-day-btn.active {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .strip-day-name {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .strip-day-num {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    font-size: 12px;
    font-weight: 650;
  }

  .strip-day-btn.today .strip-day-num {
    background: var(--accent);
    color: var(--accent-contrast);
  }

  .strip-item-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--dot-color, var(--accent));
  }

  .strip-item-dot.empty {
    background: transparent;
  }

  /* ── Continuous Multi-Week Feed ── */
  .week-feed-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 4px 0 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .week-feed-scroll::-webkit-scrollbar {
    width: 6px;
  }

  .week-feed-scroll::-webkit-scrollbar-thumb {
    background: var(--border-light);
    border-radius: 4px;
  }

  .feed-loader {
    padding: 6px;
    text-align: center;
    font-size: 10px;
    color: var(--text-tertiary);
  }

  /* ── Month Transition Banner ── */
  .month-divider {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px 4px;
  }

  .month-title {
    font-size: 12px;
    font-weight: 750;
    color: var(--accent);
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .month-divider::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  /* ── Day Group Block ── */
  .day-group {
    display: flex;
    gap: 10px;
    padding: 6px 12px;
    transition: background 0.15s;
    border-radius: var(--radius-md);
  }

  .day-group.today {
    background: color-mix(in srgb, var(--accent) 4%, transparent);
  }

  .day-group.selected {
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  .day-group.drag-over {
    background: color-mix(in srgb, var(--accent) 10%, transparent);
    outline: 2px dashed var(--accent);
    outline-offset: -2px;
  }

  /* ── Left Date Badge Column ── */
  .day-badge-col {
    flex: 0 0 54px;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 2px;
    gap: 4px;
  }

  .date-badge-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    width: 100%;
  }

  .badge-weekday {
    font-size: 9px;
    font-weight: 750;
    color: var(--text-tertiary);
    letter-spacing: 0.05em;
  }

  .badge-daynum {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    font-size: 16px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .date-badge-box.today .badge-daynum {
    background: var(--accent);
    color: var(--accent-contrast);
    box-shadow: 0 2px 6px color-mix(in srgb, var(--accent) 40%, transparent);
  }

  .badge-today-pill {
    font-size: 7.5px;
    font-weight: 800;
    color: var(--accent);
    letter-spacing: 0.04em;
    margin-top: 1px;
  }

  .day-add-btn {
    width: 22px;
    height: 22px;
    display: grid;
    place-items: center;
    border: 1px solid var(--border-light);
    border-radius: 50%;
    background: transparent;
    color: var(--text-tertiary);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
    opacity: 0.35;
    transition: opacity 0.15s, background 0.15s, color 0.15s;
  }

  .day-group:hover .day-add-btn {
    opacity: 1;
  }

  .day-add-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border);
  }

  /* ── Right Cards Stream ── */
  .day-cards-stream {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  /* Empty Day Placeholder */
  .empty-day-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 10px;
    border: 1px dashed var(--border-light);
    border-radius: var(--radius-md);
    color: var(--text-tertiary);
    font-size: 11px;
    background: transparent;
  }

  .empty-text {
    font-style: italic;
  }

  .empty-add-btn {
    border: 0;
    background: transparent;
    color: var(--accent);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .empty-add-btn:hover {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  /* ── Google Calendar Widget Card ── */
  .agenda-card {
    display: flex;
    align-items: stretch;
    border: 1px solid var(--border-light);
    border-radius: 8px;
    background: var(--bg-secondary);
    overflow: hidden;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s, box-shadow 0.12s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
  }

  .agenda-card:hover {
    background: var(--bg-hover);
    border-color: color-mix(in srgb, var(--item-color) 40%, var(--border));
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
  }

  .agenda-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .agenda-card.completed {
    opacity: 0.62;
  }

  .agenda-card.google {
    border-style: dashed;
  }

  .card-accent-bar {
    width: 5px;
    flex: 0 0 5px;
  }

  .card-body {
    flex: 1;
    min-width: 0;
    padding: 7px 10px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  /* Card Top Row */
  .card-top-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .task-checkbox {
    width: 16px;
    height: 16px;
    flex: 0 0 16px;
    border: 1.5px solid var(--item-color, var(--text-tertiary));
    border-radius: 4px;
    background: transparent;
    color: var(--accent-contrast);
    display: grid;
    place-items: center;
    cursor: pointer;
    padding: 0;
    transition: background 0.12s, border-color 0.12s;
  }

  .task-checkbox:hover {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 15%, transparent);
  }

  .task-checkbox.checked {
    background: var(--accent);
    border-color: var(--accent);
  }

  .card-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    font-weight: 650;
    color: var(--text-primary);
  }

  .completed .card-title {
    text-decoration: line-through;
    color: var(--text-secondary);
  }

  /* Badges */
  .badge-pill {
    flex: 0 0 auto;
    font-size: 9px;
    font-weight: 650;
    padding: 2px 6px;
    border-radius: 999px;
    white-space: nowrap;
  }

  .project-pill {
    display: flex;
    align-items: center;
    gap: 4px;
    background: color-mix(in srgb, var(--proj-color) 12%, transparent);
    color: var(--proj-color);
  }

  .proj-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--proj-color);
  }

  .google-pill {
    background: color-mix(in srgb, #4285f4 15%, transparent);
    color: #4285f4;
  }

  .repeat-pill {
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: 1px solid var(--border-light);
  }

  .time-pill {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
    color: var(--accent);
  }

  /* Card Meta Row */
  .card-meta-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    color: var(--text-secondary);
    font-size: 10px;
  }

  .meta-tag {
    display: flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
    flex: 0 0 auto;
  }

  .meta-tag.time {
    font-variant-numeric: tabular-nums;
  }

  .meta-tag.all-day {
    color: var(--text-tertiary);
  }

  .meta-tag.duration {
    color: var(--text-tertiary);
    background: color-mix(in srgb, var(--text-tertiary) 10%, transparent);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 9px;
  }

  .meta-desc {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-tertiary);
    font-size: 10px;
  }

  /* Compact Mode */
  :global(body.compact-mode) .sticky-week-strip {
    padding: 4px 6px;
  }

  :global(body.compact-mode) .day-group {
    padding: 4px 8px;
    gap: 8px;
  }

  :global(body.compact-mode) .card-body {
    padding: 5px 8px;
  }

  :global(body.compact-mode) .card-title {
    font-size: 11px;
  }

  @media (prefers-reduced-motion: reduce) {
    * {
      scroll-behavior: auto !important;
      transition: none !important;
    }
  }
</style>

