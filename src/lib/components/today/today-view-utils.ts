import type { CalendarEvent } from "$lib/services/db";

export function getTodayProgress(completed: number, total: number): number {
  if (total <= 0) return 0;
  return Math.min(100, Math.round((Math.max(0, completed) / total) * 100));
}

export function formatTodayDeadline(deadline: string): string | null {
  if (!deadline.includes("T")) return null;
  const time = deadline.slice(11, 16);
  return /^\d{2}:\d{2}$/.test(time) ? time : null;
}

export function sortTodayEvents(events: CalendarEvent[]): CalendarEvent[] {
  return [...events].sort((left, right) => {
    if (left.is_all_day !== right.is_all_day) return left.is_all_day ? -1 : 1;
    return left.date.localeCompare(right.date) || left.title.localeCompare(right.title);
  });
}

export function formatEventTime(event: CalendarEvent): string {
  if (event.is_all_day) return "All day";
  if (!event.date.includes("T")) return "Scheduled";
  const time = event.date.slice(11, 16);
  return /^\d{2}:\d{2}$/.test(time) ? time : "Scheduled";
}
