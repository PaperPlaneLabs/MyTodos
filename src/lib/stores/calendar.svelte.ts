import { db, type Task, type CalendarEvent } from '$lib/services/db';
import type { CalendarDay, TimeEntryWithTask } from '$lib/types/calendar';

let currentDate = $state<Date>(new Date());
let selectedDate = $state<Date | null>(null);
let viewMode = $state<'month' | 'week' | 'day'>('month');
let tasksByDate = $state<Map<string, Task[]>>(new Map());
let eventsByDate = $state<Map<string, CalendarEvent[]>>(new Map());
let timeEntriesByDate = $state<Map<string, TimeEntryWithTask[]>>(new Map());
let isLoading = $state(false);
let lastLoadedRangeKey = $state<string | null>(null);
let activeLoadId = 0;

export const calendarStore = {
    get currentDate() { return currentDate; },
    get selectedDate() { return selectedDate; },
    get viewMode() { return viewMode; },
    get tasksByDate() { return tasksByDate; },
    get eventsByDate() { return eventsByDate; },
    get timeEntriesByDate() { return timeEntriesByDate; },
    get isLoading() { return isLoading; },

    setCurrentDate(date: Date) {
        currentDate = date;
        void this.ensureCurrentRangeLoaded();
    },

    setSelectedDate(date: Date | null) {
        selectedDate = date;
        if (date) {
            currentDate = date;
            void this.ensureCurrentRangeLoaded();
        }
    },

    setViewMode(mode: 'month' | 'week' | 'day') {
        viewMode = mode;
        void this.ensureCurrentRangeLoaded();
    },

    async ensureCurrentRangeLoaded() {
        const { startDate, endDate, rangeKey } = this.getVisibleRange();
        if (rangeKey === lastLoadedRangeKey) {
            return;
        }
        await this.loadRangeData(startDate, endDate, rangeKey);
    },

    async refreshCurrentRange() {
        const { startDate, endDate, rangeKey } = this.getVisibleRange();
        await this.loadRangeData(startDate, endDate, rangeKey);
    },

    async loadRangeData(startDate: string, endDate: string, rangeKey: string) {
        const loadId = ++activeLoadId;
        isLoading = true;

        try {
            const [tasks, events, timeEntries] = await Promise.all([
                db.tasks.getByDeadlineRange(startDate, endDate),
                db.calendarEvents.getInRange(startDate, endDate),
                db.timeEntries.getWithTasks(startDate, endDate),
            ]);

            if (loadId !== activeLoadId) {
                return;
            }

            const nextTasksByDate = new Map<string, Task[]>();
            for (const task of tasks) {
                const deadline = task.deadline ?? null;
                if (deadline) {
                    const dateOnly = deadline.split('T')[0];
                    const existing = nextTasksByDate.get(dateOnly) || [];
                    existing.push(task);
                    nextTasksByDate.set(dateOnly, existing);
                }
            }

            const nextEventsByDate = new Map<string, CalendarEvent[]>();
            for (const event of events) {
                const existing = nextEventsByDate.get(event.date) || [];
                existing.push(event);
                nextEventsByDate.set(event.date, existing);
            }

            const nextTimeEntriesByDate = new Map<string, TimeEntryWithTask[]>();
            for (const entry of timeEntries) {
                const entryDate = this.dateToString(new Date(entry.started_at * 1000));
                const existing = nextTimeEntriesByDate.get(entryDate) || [];
                existing.push(entry);
                nextTimeEntriesByDate.set(entryDate, existing);
            }

            tasksByDate = nextTasksByDate;
            eventsByDate = nextEventsByDate;
            timeEntriesByDate = nextTimeEntriesByDate;
            lastLoadedRangeKey = rangeKey;
        } finally {
            if (loadId === activeLoadId) {
                isLoading = false;
            }
        }
    },

    async updateTaskDeadline(taskId: number, deadline: string | null) {
        await db.tasks.updateDeadline(taskId, deadline);
        await this.refreshCurrentRange();
    },

    getTasksForDate(date: string): Task[] {
        return tasksByDate.get(date) || [];
    },

    getEventsForDate(date: string): CalendarEvent[] {
        return eventsByDate.get(date) || [];
    },

    getTimeEntriesForDate(date: string): TimeEntryWithTask[] {
        return timeEntriesByDate.get(date) || [];
    },

    getVisibleRange(): { startDate: string; endDate: string; rangeKey: string } {
        let start: Date;
        let end: Date;

        if (viewMode === 'week') {
            start = this.getWeekStart(currentDate);
            end = new Date(start);
            end.setDate(start.getDate() + 6);
        } else if (viewMode === 'day') {
            start = new Date(currentDate);
            start.setHours(0, 0, 0, 0);
            end = new Date(start);
        } else {
            const year = currentDate.getFullYear();
            const month = currentDate.getMonth();
            const firstDay = new Date(year, month, 1);
            const lastDay = new Date(year, month + 1, 0);

            start = new Date(firstDay);
            start.setDate(firstDay.getDate() - firstDay.getDay());
            start.setHours(0, 0, 0, 0);

            end = new Date(lastDay);
            end.setDate(lastDay.getDate() + (6 - lastDay.getDay()));
            end.setHours(0, 0, 0, 0);
        }

        const startDate = this.dateToString(start);
        const endDate = this.dateToString(end);
        return {
            startDate,
            endDate,
            rangeKey: `${viewMode}:${startDate}:${endDate}`,
        };
    },

    generateCalendarDays(year: number, month: number): CalendarDay[] {
        const days: CalendarDay[] = [];
        const firstDay = new Date(year, month, 1);
        const lastDay = new Date(year, month + 1, 0);
        const today = new Date();
        const todayStr = this.dateToString(today);

        const selectedDateStr = selectedDate ? this.dateToString(selectedDate) : null;

        const startPadding = firstDay.getDay();
        const prevMonth = new Date(year, month, 0);

        for (let i = startPadding - 1; i >= 0; i--) {
            const date = new Date(year, month - 1, prevMonth.getDate() - i);
            const dateStr = this.dateToString(date);
            days.push({
                date,
                isCurrentMonth: false,
                isToday: dateStr === todayStr,
                isSelected: dateStr === selectedDateStr,
                tasks: this.getTasksForDate(dateStr),
                events: this.getEventsForDate(dateStr),
                timeEntries: this.getTimeEntriesForDate(dateStr),
            });
        }

        for (let i = 1; i <= lastDay.getDate(); i++) {
            const date = new Date(year, month, i);
            const dateStr = this.dateToString(date);
            days.push({
                date,
                isCurrentMonth: true,
                isToday: dateStr === todayStr,
                isSelected: dateStr === selectedDateStr,
                tasks: this.getTasksForDate(dateStr),
                events: this.getEventsForDate(dateStr),
                timeEntries: this.getTimeEntriesForDate(dateStr),
            });
        }

        const endPadding = 42 - days.length;
        for (let i = 1; i <= endPadding; i++) {
            const date = new Date(year, month + 1, i);
            const dateStr = this.dateToString(date);
            days.push({
                date,
                isCurrentMonth: false,
                isToday: dateStr === todayStr,
                isSelected: dateStr === selectedDateStr,
                tasks: this.getTasksForDate(dateStr),
                events: this.getEventsForDate(dateStr),
                timeEntries: this.getTimeEntriesForDate(dateStr),
            });
        }

        return days;
    },

    generateWeekDays(weekStart: Date): { date: Date; dayName: string }[] {
        const days: { date: Date; dayName: string }[] = [];
        const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

        for (let i = 0; i < 7; i++) {
            const date = new Date(weekStart);
            date.setDate(weekStart.getDate() + i);
            days.push({
                date,
                dayName: dayNames[date.getDay()],
            });
        }

        return days;
    },

    getWeekStart(date: Date): Date {
        const d = new Date(date);
        const day = d.getDay();
        d.setDate(d.getDate() - day);
        d.setHours(0, 0, 0, 0);
        return d;
    },

    dateToString(date: Date): string {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const day = String(date.getDate()).padStart(2, '0');
        return `${year}-${month}-${day}`;
    },
};
