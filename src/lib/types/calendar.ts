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
    deadline: string | null | undefined;
}

export interface CalendarDay {
    date: Date;
    isCurrentMonth: boolean;
    isToday: boolean;
    isSelected: boolean;
    tasks: CalendarTask[];
    events: CalendarEvent[];
}
