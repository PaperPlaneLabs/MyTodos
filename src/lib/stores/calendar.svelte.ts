import { db, type Task, type CalendarEvent } from '$lib/services/db';
import type { CalendarDay, TimeEntryWithTask } from '$lib/types/calendar';

let currentDate = $state<Date>(new Date());
let selectedDate = $state<Date | null>(null);
let viewMode = $state<'month' | 'week' | 'day'>('month');
let tasksByDate = $state<Map<string, Task[]>>(new Map());
let eventsByDate = $state<Map<string, CalendarEvent[]>>(new Map());
let timeEntriesByDate = $state<Map<string, TimeEntryWithTask[]>>(new Map());
let isCalendarOpen = $state(false);
let isLoading = $state(false);

export const calendarStore = {
    get currentDate() { return currentDate; },
    get selectedDate() { return selectedDate; },
    get viewMode() { return viewMode; },
    get tasksByDate() { return tasksByDate; },
    get eventsByDate() { return eventsByDate; },
    get timeEntriesByDate() { return timeEntriesByDate; },
    get isOpen() { return isCalendarOpen; },
    get isLoading() { return isLoading; },

    open() {
        isCalendarOpen = true;
        this.loadMonthData();
    },

    close() { isCalendarOpen = false; },

    toggle() {
        if (isCalendarOpen) {
            this.close();
        } else {
            this.open();
        }
    },

    setCurrentDate(date: Date) {
        currentDate = date;
        this.loadMonthData();
    },

    setSelectedDate(date: Date | null) { selectedDate = date; },

    setViewMode(mode: 'month' | 'week' | 'day') { viewMode = mode; },

    async loadMonthData() {
        isLoading = true;
        const year = currentDate.getFullYear();
        const month = currentDate.getMonth();
        const startDate = `${year}-${String(month + 1).padStart(2, '0')}-01`;
        const endDate = `${year}-${String(month + 1).padStart(2, '0')}-31`;

        try {
            const [tasks, events, timeEntries] = await Promise.all([
                db.tasks.getByDeadlineRange(startDate, endDate),
                db.calendarEvents.getInRange(startDate, endDate),
                db.timeEntries.getWithTasks(startDate, endDate),
            ]);

            tasksByDate = new Map();
            for (const task of tasks) {
                const deadline = task.deadline ?? null;
                if (deadline) {
                    const existing = tasksByDate.get(deadline) || [];
                    existing.push(task);
                    tasksByDate.set(deadline, existing);
                }
            }

            eventsByDate = new Map();
            for (const event of events) {
                const existing = eventsByDate.get(event.date) || [];
                existing.push(event);
                eventsByDate.set(event.date, existing);
            }

            timeEntriesByDate = new Map();
            for (const entry of timeEntries) {
                const entryDate = new Date(entry.started_at * 1000).toISOString().split('T')[0];
                const existing = timeEntriesByDate.get(entryDate) || [];
                existing.push(entry);
                timeEntriesByDate.set(entryDate, existing);
            }
        } finally {
            isLoading = false;
        }
    },

    async updateTaskDeadline(taskId: number, deadline: string | null) {
        await db.tasks.updateDeadline(taskId, deadline);
        await this.loadMonthData();
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
