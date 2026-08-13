import type {
  CalendarEvent,
  GoogleCalendarEvent,
  Task,
  TimeEntryWithTask,
} from "$lib/services/db";

export type CalendarViewMode = "month" | "week";
export type CalendarSource = "tasks" | "local" | "google" | "time";
export type WeekStart = "monday" | "sunday";

export interface CalendarSourceFilters {
  tasks: boolean;
  local: boolean;
  google: boolean;
  time: boolean;
}

export interface CalendarItemBase {
  key: string;
  source: CalendarSource;
  title: string;
  description: string | null;
  color: string;
  isAllDay: boolean;
  startDate: string;
  endDate: string;
  startTime: string | null;
  endTime: string | null;
  startAt: number | null;
  endAt: number | null;
  readOnly: boolean;
}

export interface TaskCalendarItem extends CalendarItemBase {
  kind: "task";
  source: "tasks";
  task: Task;
}

export interface LocalEventCalendarItem extends CalendarItemBase {
  kind: "local_event";
  source: "local";
  event: CalendarEvent;
}

export interface GoogleEventCalendarItem extends CalendarItemBase {
  kind: "google_event";
  source: "google";
  event: GoogleCalendarEvent;
}

export interface TimeEntryCalendarItem extends CalendarItemBase {
  kind: "time_entry";
  source: "time";
  entry: TimeEntryWithTask;
}

export type CalendarItem =
  | TaskCalendarItem
  | LocalEventCalendarItem
  | GoogleEventCalendarItem
  | TimeEntryCalendarItem;

export interface CalendarDay {
  date: Date;
  dateKey: string;
  isCurrentMonth: boolean;
  isToday: boolean;
  isSelected: boolean;
  items: CalendarItem[];
}

export type RecurrenceFrequency = "daily" | "weekly" | "monthly" | "yearly";

export interface RecurrenceRule {
  frequency: RecurrenceFrequency;
  interval: number;
  weekdays: number[];
  until?: string | null;
  count?: number | null;
}

export interface PositionedCalendarItem {
  item: CalendarItem;
  top: number;
  height: number;
  column: number;
  columnCount: number;
}

export interface EventEditorDraft {
  id: number | null;
  title: string;
  description: string;
  isAllDay: boolean;
  startDate: string;
  endDate: string;
  startTime: string;
  endTime: string;
  color: string;
  recurrence: RecurrenceRule | null;
}

export type { CalendarEvent, GoogleCalendarEvent, TimeEntryWithTask };
