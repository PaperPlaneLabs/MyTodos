import type { Task } from '$lib/services/db';

export interface CalendarEvent {
    id: number;
    title: string;
    description: string | null;
    date: string;
    is_all_day: boolean;
    color: string;
}

export interface CalendarTask extends Task {
    deadline?: string | null | undefined;
}

export interface CalendarDay {
    date: Date;
    isCurrentMonth: boolean;
    isToday: boolean;
    isSelected: boolean;
    tasks: CalendarTask[];
    events: CalendarEvent[];
    timeEntries: TimeEntryWithTask[];
}

export interface TimeEntryWithTask {
    id: number;
    task_id: number;
    task_title: string;
    project_id: number | null;
    project_name: string | null;
    project_color: string | null;
    duration_seconds: number;
    started_at: number;
    ended_at: number;
    note: string | null;
}
