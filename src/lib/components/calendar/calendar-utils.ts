import type {
  CalendarEvent,
  GoogleCalendarEvent,
  Task,
  TimeEntryWithTask,
} from "$lib/services/db";
import type {
  CalendarItem,
  PositionedCalendarItem,
  WeekStart,
} from "$lib/types/calendar";

export function dateToKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function addDays(dateKey: string, days: number): string {
  const date = new Date(`${dateKey}T12:00:00`);
  date.setDate(date.getDate() + days);
  return dateToKey(date);
}

export function getWeekStart(date: Date, weekStart: WeekStart): Date {
  const result = new Date(date);
  const day = result.getDay();
  const offset = weekStart === "monday" ? (day + 6) % 7 : day;
  result.setDate(result.getDate() - offset);
  result.setHours(0, 0, 0, 0);
  return result;
}

export function getWeekDays(date: Date, weekStart: WeekStart): Date[] {
  const start = getWeekStart(date, weekStart);
  return Array.from({ length: 7 }, (_, index) => {
    const day = new Date(start);
    day.setDate(start.getDate() + index);
    return day;
  });
}

export function timeToMinutes(time: string | null): number | null {
  if (!time) return null;
  const [hours, minutes] = time.split(":").map(Number);
  if (!Number.isFinite(hours) || !Number.isFinite(minutes)) return null;
  return hours * 60 + minutes;
}

export function minutesToTime(minutes: number): string {
  const normalized = Math.max(0, Math.min(1439, Math.round(minutes)));
  return `${String(Math.floor(normalized / 60)).padStart(2, "0")}:${String(normalized % 60).padStart(2, "0")}`;
}

export function taskToCalendarItem(task: Task, color: string): CalendarItem | null {
  if (!task.deadline) return null;
  const [startDate, rawTime] = task.deadline.split("T");
  const startTime = rawTime?.slice(0, 5) ?? null;
  const duration = task.planned_duration_minutes ?? 30;
  const start = startTime ? new Date(`${startDate}T${startTime}:00`) : null;
  const end = start ? new Date(start.getTime() + duration * 60_000) : null;
  return {
    key: `task:${task.id}`,
    kind: "task",
    source: "tasks",
    title: task.title,
    description: task.description ?? null,
    color,
    isAllDay: !startTime,
    startDate,
    endDate: startTime && end ? addDays(dateToKey(end), 1) : addDays(startDate, 1),
    startTime,
    endTime: end ? `${String(end.getHours()).padStart(2, "0")}:${String(end.getMinutes()).padStart(2, "0")}` : null,
    startAt: start ? Math.floor(start.getTime() / 1000) : null,
    endAt: end ? Math.floor(end.getTime() / 1000) : null,
    readOnly: false,
    task,
  };
}

export function localEventToCalendarItem(event: CalendarEvent): CalendarItem {
  const startDate = event.start_date ?? event.date.split("T")[0];
  const startTime = event.start_time ?? (event.date.includes("T") ? event.date.split("T")[1]?.slice(0, 5) ?? null : null);
  return {
    key: `local:${event.occurrence_key ?? `${event.id}:${startDate}`}`,
    kind: "local_event",
    source: "local",
    title: event.title,
    description: event.description,
    color: event.color || "var(--accent)",
    isAllDay: event.is_all_day,
    startDate,
    endDate: event.end_date ?? addDays(startDate, 1),
    startTime,
    endTime: event.end_time ?? null,
    startAt: event.start_at ?? null,
    endAt: event.end_at ?? null,
    readOnly: false,
    event,
  };
}

export function googleEventToCalendarItem(event: GoogleCalendarEvent): CalendarItem {
  const start = event.start_at ? new Date(event.start_at * 1000) : null;
  const end = event.end_at ? new Date(event.end_at * 1000) : null;
  return {
    key: `google:${event.external_id}`,
    kind: "google_event",
    source: "google",
    title: event.title,
    description: event.description,
    color: event.color || "#4285f4",
    isAllDay: event.is_all_day,
    startDate: event.start_date,
    endDate: event.end_date,
    startTime: start ? minutesToTime(start.getHours() * 60 + start.getMinutes()) : null,
    endTime: end ? minutesToTime(end.getHours() * 60 + end.getMinutes()) : null,
    startAt: event.start_at,
    endAt: event.end_at,
    readOnly: true,
    event,
  };
}

export function timeEntryToCalendarItem(entry: TimeEntryWithTask): CalendarItem {
  const start = new Date(entry.started_at * 1000);
  const end = new Date(entry.ended_at * 1000);
  return {
    key: `time:${entry.id}`,
    kind: "time_entry",
    source: "time",
    title: entry.task_title,
    description: entry.note,
    color: entry.project_color || "var(--text-tertiary)",
    isAllDay: false,
    startDate: dateToKey(start),
    endDate: addDays(dateToKey(end), 1),
    startTime: minutesToTime(start.getHours() * 60 + start.getMinutes()),
    endTime: minutesToTime(end.getHours() * 60 + end.getMinutes()),
    startAt: entry.started_at,
    endAt: entry.ended_at,
    readOnly: false,
    entry,
  };
}

export function itemOccursOnDate(item: CalendarItem, dateKey: string): boolean {
  return item.startDate <= dateKey && dateKey < item.endDate;
}

export function formatTaskDeadline(dateKey: string, time: string | null): string {
  return time ? `${dateKey}T${time}:00` : dateKey;
}

export function positionTimedItems(
  items: CalendarItem[],
  pixelsPerMinute: number,
): PositionedCalendarItem[] {
  const candidates = items
    .map((item) => {
      const start = timeToMinutes(item.startTime);
      const end = timeToMinutes(item.endTime);
      if (start === null) return null;
      return {
        item,
        start,
        end: end !== null && end > start ? end : start + 30,
      };
    })
    .filter((value): value is NonNullable<typeof value> => value !== null)
    .sort((left, right) => left.start - right.start || right.end - left.end);

  const result: PositionedCalendarItem[] = [];
  let cluster: typeof candidates = [];
  let clusterEnd = -1;
  const flush = () => {
    if (cluster.length === 0) return;
    const columnEnds: number[] = [];
    const assignments = cluster.map((candidate) => {
      let column = columnEnds.findIndex((end) => end <= candidate.start);
      if (column === -1) {
        column = columnEnds.length;
        columnEnds.push(candidate.end);
      } else {
        columnEnds[column] = candidate.end;
      }
      return { candidate, column };
    });
    const columnCount = Math.max(1, columnEnds.length);
    for (const { candidate, column } of assignments) {
      result.push({
        item: candidate.item,
        top: candidate.start * pixelsPerMinute,
        height: Math.max(22, (candidate.end - candidate.start) * pixelsPerMinute),
        column,
        columnCount,
      });
    }
    cluster = [];
    clusterEnd = -1;
  };

  for (const candidate of candidates) {
    if (cluster.length > 0 && candidate.start >= clusterEnd) flush();
    cluster.push(candidate);
    clusterEnd = Math.max(clusterEnd, candidate.end);
  }
  flush();
  return result;
}
