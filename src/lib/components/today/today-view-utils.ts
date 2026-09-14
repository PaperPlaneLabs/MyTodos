import type { CalendarEvent, TodayTask } from "$lib/services/db";

export type TodayAgendaItem =
  | { kind: "task"; id: number; title: string; time: string | null; isAllDay: false; color: string | null; task: TodayTask }
  | { kind: "event"; id: number; title: string; time: string | null; isAllDay: boolean; color: string; event: CalendarEvent };

export interface TodayAgenda {
  allDay: Extract<TodayAgendaItem, { kind: "event" }>[];
  anytime: Extract<TodayAgendaItem, { kind: "task" }>[];
  timeline: TodayAgendaItem[];
}

export interface UpcomingTaskGroup {
  date: string;
  label: string;
  tasks: TodayTask[];
}

export function getTodayProgress(completed: number, total: number): number {
  if (total <= 0) return 0;
  return Math.min(100, Math.round((Math.max(0, completed) / total) * 100));
}

export function formatTodayDeadline(deadline: string): string | null {
  if (!deadline.includes("T")) return null;
  const time = deadline.slice(11, 16);
  return /^\d{2}:\d{2}$/.test(time) ? time : null;
}

function compareAgendaItems(left: TodayAgendaItem, right: TodayAgendaItem): number {
  const byTime = (left.time ?? "").localeCompare(right.time ?? "");
  if (byTime !== 0) return byTime;
  if (left.kind !== right.kind) return left.kind === "event" ? -1 : 1;
  return left.title.localeCompare(right.title) || left.id - right.id;
}

export function buildTodayAgenda(tasks: TodayTask[], events: CalendarEvent[]): TodayAgenda {
  const taskItems: Extract<TodayAgendaItem, { kind: "task" }>[] = tasks.map((task) => ({
    kind: "task", id: task.id, title: task.title, time: formatTodayDeadline(task.deadline),
    isAllDay: false, color: task.project_color ?? null, task,
  }));
  const eventItems: Extract<TodayAgendaItem, { kind: "event" }>[] = events.map((event) => ({
    kind: "event", id: event.id, title: event.title,
    time: event.is_all_day ? null : formatEventTime(event), isAllDay: event.is_all_day,
    color: event.color, event,
  }));
  return {
    allDay: eventItems.filter((item) => item.isAllDay).sort(compareAgendaItems),
    anytime: taskItems.filter((item) => item.time === null).sort(compareAgendaItems),
    timeline: [...taskItems.filter((item) => item.time !== null), ...eventItems.filter((item) => !item.isAllDay)].sort(compareAgendaItems),
  };
}

function formatUpcomingDateLabel(date: string, today: string): string {
  if (date === today) return "Today";

  const tomorrow = new Date(`${today}T12:00:00`);
  tomorrow.setDate(tomorrow.getDate() + 1);
  const tomorrowKey = [
    tomorrow.getFullYear(),
    String(tomorrow.getMonth() + 1).padStart(2, "0"),
    String(tomorrow.getDate()).padStart(2, "0"),
  ].join("-");
  if (date === tomorrowKey) return "Tomorrow";

  return new Date(`${date}T12:00:00`).toLocaleDateString("en-US", {
    weekday: "long",
    month: "short",
    day: "numeric",
  });
}

export function groupUpcomingTasks(tasks: TodayTask[], today: string): UpcomingTaskGroup[] {
  const groups = new Map<string, TodayTask[]>();

  for (const task of tasks) {
    const date = task.deadline.slice(0, 10);
    const tasksForDate = groups.get(date);
    if (tasksForDate) {
      tasksForDate.push(task);
    } else {
      groups.set(date, [task]);
    }
  }

  return [...groups.entries()].map(([date, groupedTasks]) => ({
    date,
    label: formatUpcomingDateLabel(date, today),
    tasks: groupedTasks,
  }));
}

export function formatEventTime(event: CalendarEvent): string {
  if (event.is_all_day) return "All day";
  if (event.start_time && /^\d{2}:\d{2}$/.test(event.start_time)) return event.start_time;
  if (!event.date.includes("T")) return "Scheduled";
  const time = event.date.slice(11, 16);
  return /^\d{2}:\d{2}$/.test(time) ? time : "Scheduled";
}
