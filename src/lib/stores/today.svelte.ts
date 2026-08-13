import { db, type CalendarEvent, type TimeStats, type TodayTaskSummary } from "$lib/services/db";
import { TodayLoader, type TodayDataSource } from "$lib/stores/today-loader";

const defaultDataSource: TodayDataSource = {
  getTaskSummary: (todayStart, tomorrowStart) =>
    db.tasks.getTodaySummary(todayStart, tomorrowStart),
  getEvents: (startDate, endDate) =>
    db.calendarEvents.getInRange(startDate, endDate),
  getStats: () => db.timeEntries.getTimeStats(true),
};

const loader = new TodayLoader(defaultDataSource);
let taskSummary = $state<TodayTaskSummary>({
  overdue: [],
  today: [],
  completed_today: 0,
  total_today: 0,
});
let events = $state<CalendarEvent[]>([]);
let stats = $state<TimeStats | null>(null);
let date = $state("");
let loading = $state(false);
let error = $state<string | null>(null);
let activeRefreshId = 0;

export const todayStore = {
    get taskSummary() {
      return taskSummary;
    },
    get events() {
      return events;
    },
    get stats() {
      return stats;
    },
    get date() {
      return date;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },

    async refresh(now: Date = new Date()) {
      const refreshId = ++activeRefreshId;
      loading = true;
      error = null;

      try {
        const snapshot = await loader.load(now);
        if (!snapshot) return;
        taskSummary = snapshot.taskSummary;
        events = snapshot.events;
        stats = snapshot.stats;
        date = snapshot.date;
      } catch (cause) {
        if (refreshId !== activeRefreshId) return;
        error = cause instanceof Error ? cause.message : "Failed to load Today";
      } finally {
        if (refreshId === activeRefreshId) loading = false;
      }
    },
};
