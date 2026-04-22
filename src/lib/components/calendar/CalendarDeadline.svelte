<script lang="ts">
  import { calendarStore } from "$lib/stores/calendar.svelte";
  import { projectStore } from "$lib/stores/projects.svelte";
  import { uiStore } from "$lib/stores/ui.svelte";
  import type { Task } from "$lib/services/db";

  type DeadlineItem = {
    id: number;
    title: string;
    color: string;
    time: string | null;
    sortTime: string;
    isAllDay: boolean;
    task: Task;
  };

  type DeadlineGroup = {
    date: Date;
    items: DeadlineItem[];
  };

  let isPortrait = $derived(
    uiStore.windowOrientation === "left" ||
      uiStore.windowOrientation === "right",
  );

  function getProjectColor(projectId: number | null | undefined): string {
    if (!projectId) return "var(--text-tertiary)";
    const project = projectStore.projects.find((entry) => entry.id === projectId);
    return project?.color || "var(--text-tertiary)";
  }

  function getDeadlineDate(deadline: string): Date {
    const date = deadline.includes("T")
      ? new Date(deadline)
      : new Date(`${deadline}T00:00:00`);
    date.setSeconds(0, 0);
    return date;
  }

  function formatDeadlineTime(deadline: string | null | undefined): string | null {
    if (!deadline || !deadline.includes("T")) return null;
    const date = getDeadlineDate(deadline);
    return date.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
      hour12: true,
    });
  }

  function getSortableTime(deadline: string | null | undefined): string {
    if (!deadline || !deadline.includes("T")) return "23:59";
    return deadline.split("T")[1]?.slice(0, 5) ?? "23:59";
  }

  function formatDateLabel(date: Date): string {
    return date.toLocaleDateString("en-US", {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  }

  function isToday(date: Date): boolean {
    const today = new Date();
    return date.toDateString() === today.toDateString();
  }

  function openTask(task: Task) {
    uiStore.openTaskModal({ taskId: task.id });
  }

  let deadlineGroups = $derived.by(() => {
    const grouped = new Map<string, DeadlineGroup>();

    for (const task of calendarStore.deadlineTasks) {
      const deadline = task.deadline ?? null;
      if (!deadline || task.completed) continue;

      const dateOnly = deadline.split("T")[0];
      const groupDate = getDeadlineDate(dateOnly);
      const existingGroup = grouped.get(dateOnly) ?? {
        date: groupDate,
        items: [],
      };

      existingGroup.items.push({
        id: task.id,
        title: task.title,
        color: getProjectColor(task.project_id),
        time: formatDeadlineTime(deadline),
        sortTime: getSortableTime(deadline),
        isAllDay: !deadline.includes("T"),
        task,
      });

      grouped.set(dateOnly, existingGroup);
    }

    return Array.from(grouped.entries())
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([, group]) => ({
        ...group,
        items: [...group.items].sort((left, right) => {
          if (left.isAllDay !== right.isAllDay) return left.isAllDay ? 1 : -1;
          if (left.sortTime !== right.sortTime) {
            return left.sortTime.localeCompare(right.sortTime);
          }
          return left.title.localeCompare(right.title);
        }),
      }));
  });
</script>

<div class="deadline-view" class:portrait={isPortrait}>
  {#if deadlineGroups.length === 0}
    <div class="empty-state">
      <h3>No upcoming deadlines</h3>
      <p>Tasks with future deadlines will appear here in a serial list.</p>
    </div>
  {:else}
    {#each deadlineGroups as group (calendarStore.dateToString(group.date))}
      <div class="day-row" class:today={isToday(group.date)}>
        <div class="date-column">
          <span class="day-name"
            >{group.date.toLocaleDateString("en-US", { weekday: "short" })}</span
          >
          <div class="day-number-wrapper">
            <span class="day-number">{group.date.getDate()}</span>
          </div>
          <span class="date-label">{formatDateLabel(group.date)}</span>
        </div>

        <div class="items-column">
          {#each group.items as item (item.id)}
            <button
              class="deadline-item"
              class:has-time={!!item.time}
              type="button"
              onclick={() => openTask(item.task)}
              style="--item-color: {item.color}"
            >
              <div class="time-col">
                <span class="item-time" class:all-day={item.isAllDay}
                  >{item.time ?? "All Day"}</span
                >
              </div>
              <div
                class="color-bar"
                style="background-color: {item.color}"
              ></div>
              <div class="content-col">
                <span class="item-title">{item.title}</span>
              </div>
            </button>
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .deadline-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    background: var(--bg-primary);
  }

  .empty-state {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--text-secondary);
  }

  .empty-state h3 {
    margin: 0;
    font-size: 18px;
    color: var(--text-primary);
  }

  .empty-state p {
    margin: 0;
    max-width: 320px;
    line-height: 1.5;
  }

  .day-row {
    display: grid;
    grid-template-columns: 96px 1fr;
    border-bottom: 1px solid var(--border-light);
    min-height: 92px;
  }

  .day-row:last-child {
    border-bottom: none;
  }

  .date-column {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: var(--spacing-md) var(--spacing-sm);
    border-right: 1px solid var(--border-light);
    background: var(--bg-secondary);
  }

  .day-name {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .day-number-wrapper {
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
  }

  .day-number {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .date-label {
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .day-row.today .day-number-wrapper {
    background: var(--accent);
  }

  .day-row.today .day-number {
    color: var(--accent-contrast);
  }

  .day-row.today .day-name {
    color: var(--accent);
  }

  .day-row.today .items-column {
    background: color-mix(in srgb, var(--accent) 4%, transparent);
  }

  .items-column {
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .deadline-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--item-color, var(--accent)) 8%, transparent);
    border: none;
    text-align: left;
    width: 100%;
    cursor: pointer;
    transition: background 0.1s;
  }

  .deadline-item:hover {
    background: color-mix(in srgb, var(--item-color, var(--accent)) 16%, transparent);
  }

  .time-col {
    width: 68px;
    flex-shrink: 0;
    text-align: right;
  }

  .item-time {
    font-size: 12px;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .all-day {
    font-size: 10px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-weight: 700;
    letter-spacing: 0.04em;
  }

  .color-bar {
    width: 4px;
    height: 24px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .content-col {
    flex: 1;
    overflow: hidden;
    display: flex;
    align-items: center;
  }

  .item-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .deadline-view.portrait .day-row {
    grid-template-columns: 82px 1fr;
  }

  .deadline-view.portrait .date-column {
    padding-inline: 6px;
  }

  .deadline-view.portrait .time-col {
    width: 60px;
  }
  </style>
