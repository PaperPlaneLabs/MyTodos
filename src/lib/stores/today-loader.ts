import type {
  CalendarEvent,
  TimeStats,
  TodayTaskSummary,
} from "$lib/services/db";

export interface TodayDataSource {
  getTaskSummary(todayStart: string, tomorrowStart: string): Promise<TodayTaskSummary>;
  getEvents(startDate: string, endDate: string): Promise<CalendarEvent[]>;
  getStats(): Promise<TimeStats>;
}

export interface TodaySnapshot {
  taskSummary: TodayTaskSummary;
  events: CalendarEvent[];
  stats: TimeStats;
  date: string;
}

function dateToString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function getTodayDateBoundaries(now: Date = new Date()): {
  todayStart: string;
  tomorrowStart: string;
} {
  const today = new Date(now);
  today.setHours(0, 0, 0, 0);
  const tomorrow = new Date(today);
  tomorrow.setDate(tomorrow.getDate() + 1);
  return {
    todayStart: dateToString(today),
    tomorrowStart: dateToString(tomorrow),
  };
}

export class TodayLoader {
  private activeLoadId = 0;

  constructor(private readonly dataSource: TodayDataSource) {}

  async load(now: Date = new Date()): Promise<TodaySnapshot | null> {
    const loadId = ++this.activeLoadId;
    const { todayStart, tomorrowStart } = getTodayDateBoundaries(now);
    let taskSummary: TodayTaskSummary;
    let events: CalendarEvent[];
    let stats: TimeStats;
    try {
      [taskSummary, events, stats] = await Promise.all([
        this.dataSource.getTaskSummary(todayStart, tomorrowStart),
        this.dataSource.getEvents(todayStart, todayStart),
        this.dataSource.getStats(),
      ]);
    } catch (error) {
      if (loadId !== this.activeLoadId) return null;
      throw error;
    }

    if (loadId !== this.activeLoadId) return null;
    return { taskSummary, events, stats, date: todayStart };
  }
}
