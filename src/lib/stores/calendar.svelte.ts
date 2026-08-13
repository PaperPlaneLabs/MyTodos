import {
  db,
  type CalendarEvent,
  type CalendarEventInput,
  type Task,
} from "$lib/services/db";
import {
  addDays,
  dateToKey,
  formatTaskDeadline,
  getWeekDays,
  getWeekStart,
  googleEventToCalendarItem,
  itemOccursOnDate,
  localEventToCalendarItem,
  taskToCalendarItem,
  timeEntryToCalendarItem,
  timeToMinutes,
} from "$lib/components/calendar/calendar-utils";
import { projectStore } from "$lib/stores/projects.svelte";
import type {
  CalendarDay,
  CalendarItem,
  CalendarSource,
  CalendarSourceFilters,
  CalendarViewMode,
  EventEditorDraft,
  WeekStart,
} from "$lib/types/calendar";

const DEFAULT_FILTERS: CalendarSourceFilters = {
  tasks: true,
  local: true,
  google: true,
  time: false,
};

let currentDate = $state<Date>(new Date());
let selectedDate = $state<Date | null>(null);
let selectedItemKey = $state<string | null>(null);
let viewMode = $state<CalendarViewMode>("month");
let weekStartPreference = $state<WeekStart>("monday");
let filters = $state<CalendarSourceFilters>({ ...DEFAULT_FILTERS });
let allItems = $state<CalendarItem[]>([]);
let rawTasks = $state<Task[]>([]);
let isLoading = $state(false);
let error = $state<string | null>(null);
let googleIsStale = $state(false);
let googleError = $state<string | null>(null);
let editorDraft = $state<EventEditorDraft | null>(null);
let lastLoadedRangeKey = $state<string | null>(null);
let activeLoadId = 0;

function projectColor(projectId: number | null | undefined): string {
  if (!projectId) return "var(--text-tertiary)";
  return projectStore.projects.find((project) => project.id === projectId)?.color
    ?? "var(--text-tertiary)";
}

function sourceEnabled(source: CalendarSource): boolean {
  return filters[source];
}

function savePreferences(): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem("calendarWeekStart", weekStartPreference);
  localStorage.setItem("calendarSourceFilters", JSON.stringify(filters));
}

function eventInputFromDraft(draft: EventEditorDraft): CalendarEventInput {
  const startTime = draft.isAllDay ? null : draft.startTime;
  const endTime = draft.isAllDay ? null : draft.endTime;
  const start = startTime ? new Date(`${draft.startDate}T${startTime}:00`) : null;
  let end = endTime ? new Date(`${draft.endDate}T${endTime}:00`) : null;
  if (start && end && end <= start) {
    end = new Date(end);
    end.setDate(end.getDate() + 1);
  }
  const lastDate = end ? dateToKey(end) : draft.endDate;
  return {
    title: draft.title.trim(),
    description: draft.description.trim() || null,
    is_all_day: draft.isAllDay,
    color: draft.color,
    start_date: draft.startDate,
    end_date: addDays(lastDate, 1),
    start_time: startTime,
    end_time: endTime,
    start_at: start ? Math.floor(start.getTime() / 1000) : null,
    end_at: end ? Math.floor(end.getTime() / 1000) : null,
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    recurrence_rule: draft.recurrence ? JSON.stringify(draft.recurrence) : null,
  };
}

function eventInputFromEvent(event: CalendarEvent): CalendarEventInput {
  return {
    title: event.title,
    description: event.description,
    is_all_day: event.is_all_day,
    color: event.color,
    start_date: event.start_date ?? event.date.split("T")[0],
    end_date: event.end_date ?? addDays(event.date.split("T")[0], 1),
    start_time: event.start_time ?? null,
    end_time: event.end_time ?? null,
    start_at: event.start_at ?? null,
    end_at: event.end_at ?? null,
    timezone: event.timezone ?? null,
    recurrence_rule: event.recurrence_rule ?? null,
  };
}

export const calendarStore = {
  get currentDate() { return currentDate; },
  get selectedDate() { return selectedDate; },
  get selectedItemKey() { return selectedItemKey; },
  get selectedItem() {
    return selectedItemKey
      ? allItems.find((item) => item.key === selectedItemKey) ?? null
      : null;
  },
  get viewMode() { return viewMode; },
  get weekStart() { return weekStartPreference; },
  get filters() { return filters; },
  get allItems() { return allItems; },
  get visibleItems() { return allItems.filter((item) => sourceEnabled(item.source)); },
  get isLoading() { return isLoading; },
  get error() { return error; },
  get googleIsStale() { return googleIsStale; },
  get googleError() { return googleError; },
  get editorDraft() { return editorDraft; },
  get inspectorOpen() { return selectedDate !== null || selectedItemKey !== null; },

  initPreferences() {
    if (typeof localStorage === "undefined") return;
    const savedWeekStart = localStorage.getItem("calendarWeekStart");
    if (savedWeekStart === "monday" || savedWeekStart === "sunday") {
      weekStartPreference = savedWeekStart;
    }
    const savedFilters = localStorage.getItem("calendarSourceFilters");
    if (savedFilters) {
      try {
        filters = { ...DEFAULT_FILTERS, ...JSON.parse(savedFilters) };
      } catch {
        filters = { ...DEFAULT_FILTERS };
      }
    }
  },

  setCurrentDate(date: Date) {
    currentDate = new Date(date);
    void this.ensureCurrentRangeLoaded();
  },

  setSelectedDate(date: Date | null) {
    selectedDate = date ? new Date(date) : null;
    selectedItemKey = null;
    if (date) {
      currentDate = new Date(date);
      void this.ensureCurrentRangeLoaded();
    }
  },

  selectItem(item: CalendarItem | null) {
    selectedItemKey = item?.key ?? null;
    if (item) selectedDate = new Date(`${item.startDate}T12:00:00`);
  },

  closeInspector() {
    selectedItemKey = null;
    selectedDate = null;
  },

  openNewEvent(date?: string, time?: string | null) {
    const startDate = date ?? (selectedDate ? dateToKey(selectedDate) : dateToKey(currentDate));
    const now = new Date();
    const roundedMinutes = Math.min(23 * 60 + 30, Math.ceil((now.getHours() * 60 + now.getMinutes()) / 30) * 30);
    const startTime = time ?? `${String(Math.floor(roundedMinutes / 60)).padStart(2, "0")}:${String(roundedMinutes % 60).padStart(2, "0")}`;
    const endMinutes = Math.min(23 * 60 + 59, (timeToMinutes(startTime) ?? 9 * 60) + 60);
    editorDraft = {
      id: null,
      title: "",
      description: "",
      isAllDay: time === null,
      startDate,
      endDate: startDate,
      startTime,
      endTime: `${String(Math.floor(endMinutes / 60)).padStart(2, "0")}:${String(endMinutes % 60).padStart(2, "0")}`,
      color: "#6366f1",
      recurrence: null,
    };
    selectedDate = new Date(`${startDate}T12:00:00`);
    selectedItemKey = null;
  },

  editEvent(event: CalendarEvent) {
    const start = event.start_at ? new Date(event.start_at * 1000) : null;
    const end = event.end_at ? new Date(event.end_at * 1000) : null;
    editorDraft = {
      id: event.series_id ?? event.id,
      title: event.title,
      description: event.description ?? "",
      isAllDay: event.is_all_day,
      startDate: event.series_start_date ?? event.start_date ?? event.date.split("T")[0],
      endDate: event.is_all_day
        ? addDays(event.series_end_date ?? event.end_date ?? addDays(event.date.split("T")[0], 1), -1)
        : event.series_end_date
          ? addDays(event.series_end_date, -1)
          : end ? dateToKey(end) : event.start_date ?? event.date.split("T")[0],
      startTime: event.start_time ?? (start ? `${String(start.getHours()).padStart(2, "0")}:${String(start.getMinutes()).padStart(2, "0")}` : "09:00"),
      endTime: event.end_time ?? (end ? `${String(end.getHours()).padStart(2, "0")}:${String(end.getMinutes()).padStart(2, "0")}` : "10:00"),
      color: event.color || "#6366f1",
      recurrence: event.recurrence_rule ? JSON.parse(event.recurrence_rule) : null,
    };
  },

  closeEventEditor() {
    editorDraft = null;
  },

  setViewMode(mode: CalendarViewMode) {
    viewMode = mode;
    void this.ensureCurrentRangeLoaded();
  },

  setWeekStart(value: WeekStart) {
    weekStartPreference = value;
    lastLoadedRangeKey = null;
    savePreferences();
    void this.ensureCurrentRangeLoaded();
  },

  toggleSource(source: CalendarSource) {
    filters = { ...filters, [source]: !filters[source] };
    savePreferences();
  },

  async ensureCurrentRangeLoaded() {
    const { startDate, endDate, rangeKey } = this.getVisibleRange();
    if (rangeKey === lastLoadedRangeKey) return;
    await this.loadRangeData(startDate, endDate, rangeKey);
  },

  async refreshCurrentRange() {
    const { startDate, endDate, rangeKey } = this.getVisibleRange();
    await this.loadRangeData(startDate, endDate, rangeKey);
  },

  async loadRangeData(startDate: string, endDate: string, rangeKey: string) {
    const loadId = ++activeLoadId;
    isLoading = true;
    error = null;
    try {
      const [tasks, events, timeEntries, googleRange] = await Promise.all([
        db.tasks.getByDeadlineRange(startDate, `${endDate}T23:59:59`),
        db.calendarEvents.getInRange(startDate, endDate),
        db.timeEntries.getWithTasks(startDate, endDate),
        db.googleCalendar.getEventsInRange(startDate, endDate),
      ]);
      if (loadId !== activeLoadId) return;
      rawTasks = tasks;
      const taskItems = tasks
        .map((task) => taskToCalendarItem(task, projectColor(task.project_id)))
        .filter((item): item is CalendarItem => item !== null);
      allItems = [
        ...taskItems,
        ...events.map(localEventToCalendarItem),
        ...googleRange.events.map(googleEventToCalendarItem),
        ...timeEntries.map(timeEntryToCalendarItem),
      ].sort((left, right) =>
        left.startDate.localeCompare(right.startDate)
        || Number(right.isAllDay) - Number(left.isAllDay)
        || (left.startTime ?? "").localeCompare(right.startTime ?? "")
        || left.title.localeCompare(right.title)
      );
      googleIsStale = googleRange.stale;
      googleError = googleRange.error ?? null;
      lastLoadedRangeKey = rangeKey;
      if (selectedItemKey && !allItems.some((item) => item.key === selectedItemKey)) {
        selectedItemKey = null;
      }
    } catch (caught) {
      if (loadId !== activeLoadId) return;
      error = caught instanceof Error ? caught.message : String(caught);
      console.error("Failed to load calendar range:", caught);
    } finally {
      if (loadId === activeLoadId) isLoading = false;
    }
  },

  async updateTaskSchedule(
    taskId: number,
    deadline: string | null,
    durationMinutes: number | null,
  ) {
    await db.tasks.updateSchedule(taskId, deadline, durationMinutes);
    await this.refreshCurrentRange();
  },

  async updateTaskDeadline(taskId: number, deadline: string | null) {
    const task = rawTasks.find((candidate) => candidate.id === taskId);
    await this.updateTaskSchedule(
      taskId,
      deadline,
      task?.planned_duration_minutes ?? null,
    );
  },

  async toggleTask(taskId: number) {
    await db.tasks.toggleCompletion(taskId);
    await this.refreshCurrentRange();
  },

  async saveEvent(draft: EventEditorDraft) {
    const input = eventInputFromDraft(draft);
    if (draft.id === null) await db.calendarEvents.create(input);
    else await db.calendarEvents.update(draft.id, input);
    editorDraft = null;
    await this.refreshCurrentRange();
  },

  async deleteEvent(eventId: number) {
    await db.calendarEvents.delete(eventId);
    selectedItemKey = null;
    await this.refreshCurrentRange();
  },

  async deleteTimeEntry(entryId: number) {
    await db.timeEntries.delete(entryId);
    selectedItemKey = null;
    await this.refreshCurrentRange();
  },

  async updateTimeEntry(entryId: number, durationSeconds: number, note: string | null) {
    await db.timeEntries.update(entryId, durationSeconds, note);
    await this.refreshCurrentRange();
  },

  async rescheduleItem(item: CalendarItem, targetDate: string, targetTime: string | null) {
    if (item.kind === "task") {
      const time = targetTime ?? item.startTime;
      await this.updateTaskSchedule(
        item.task.id,
        formatTaskDeadline(targetDate, time),
        time ? item.task.planned_duration_minutes ?? 30 : null,
      );
      return;
    }
    if (item.kind !== "local_event" || item.event.recurrence_rule) return;
    const input = eventInputFromEvent(item.event);
    const originalStart = new Date(`${item.startDate}T12:00:00`);
    const target = new Date(`${targetDate}T12:00:00`);
    const shiftDays = Math.round((target.getTime() - originalStart.getTime()) / 86_400_000);
    input.start_date = targetDate;
    input.end_date = addDays(input.end_date, shiftDays);
    input.start_time = targetTime ?? input.start_time;
    if (input.start_time && input.end_time) {
      const duration = Math.max(
        5,
        (timeToMinutes(input.end_time) ?? 0) - (timeToMinutes(item.startTime) ?? 0),
      );
      const startMinutes = timeToMinutes(input.start_time) ?? 0;
      const endMinutes = startMinutes + duration;
      input.end_time = `${String(Math.floor((endMinutes % 1440) / 60)).padStart(2, "0")}:${String(endMinutes % 60).padStart(2, "0")}`;
      const start = new Date(`${targetDate}T${input.start_time}:00`);
      const end = new Date(start.getTime() + duration * 60_000);
      input.start_at = Math.floor(start.getTime() / 1000);
      input.end_at = Math.floor(end.getTime() / 1000);
      input.end_date = addDays(dateToKey(end), 1);
    }
    await db.calendarEvents.update(item.event.series_id ?? item.event.id, input);
    await this.refreshCurrentRange();
  },

  async resizeItem(item: CalendarItem, durationMinutes: number) {
    const duration = Math.max(15, Math.min(1440, Math.round(durationMinutes / 15) * 15));
    if (item.kind === "task") {
      await this.updateTaskSchedule(item.task.id, item.task.deadline ?? null, duration);
      return;
    }
    if (item.kind !== "local_event" || !item.startTime) return;
    const input = eventInputFromEvent(item.event);
    const start = new Date(`${item.startDate}T${item.startTime}:00`);
    const end = new Date(start.getTime() + duration * 60_000);
    input.end_time = `${String(end.getHours()).padStart(2, "0")}:${String(end.getMinutes()).padStart(2, "0")}`;
    input.end_at = Math.floor(end.getTime() / 1000);
    input.end_date = addDays(dateToKey(end), 1);
    await db.calendarEvents.update(item.event.series_id ?? item.event.id, input);
    await this.refreshCurrentRange();
  },

  getItemsForDate(date: string): CalendarItem[] {
    return this.visibleItems.filter((item) => itemOccursOnDate(item, date));
  },

  getTasksForDate(date: string): Task[] {
    return rawTasks.filter((task) => task.deadline?.startsWith(date));
  },

  getEventsForDate(date: string): CalendarEvent[] {
    return this.visibleItems
      .filter((item) => item.kind === "local_event" && itemOccursOnDate(item, date))
      .map((item) => (item as Extract<CalendarItem, { kind: "local_event" }>).event);
  },

  getVisibleRange(): { startDate: string; endDate: string; rangeKey: string } {
    let start: Date;
    let end: Date;
    if (viewMode === "week") {
      start = getWeekStart(currentDate, weekStartPreference);
      end = new Date(start);
      end.setDate(start.getDate() + 6);
    } else {
      const first = new Date(currentDate.getFullYear(), currentDate.getMonth(), 1);
      start = getWeekStart(first, weekStartPreference);
      end = new Date(start);
      end.setDate(start.getDate() + 41);
    }
    const startDate = dateToKey(start);
    const endDate = dateToKey(end);
    return {
      startDate,
      endDate,
      rangeKey: `${viewMode}:${weekStartPreference}:${startDate}:${endDate}`,
    };
  },

  generateCalendarDays(year: number, month: number): CalendarDay[] {
    const first = new Date(year, month, 1);
    const start = getWeekStart(first, weekStartPreference);
    const today = dateToKey(new Date());
    const selected = selectedDate ? dateToKey(selectedDate) : null;
    return Array.from({ length: 42 }, (_, index) => {
      const date = new Date(start);
      date.setDate(start.getDate() + index);
      const dateKey = dateToKey(date);
      return {
        date,
        dateKey,
        isCurrentMonth: date.getMonth() === month,
        isToday: dateKey === today,
        isSelected: dateKey === selected,
        items: this.getItemsForDate(dateKey),
      };
    });
  },

  generateWeekDays(date: Date): { date: Date; dayName: string }[] {
    return getWeekDays(date, weekStartPreference).map((day) => ({
      date: day,
      dayName: day.toLocaleDateString("en-US", { weekday: "short" }),
    }));
  },

  getWeekStart(date: Date): Date {
    return getWeekStart(date, weekStartPreference);
  },

  dateToString(date: Date): string {
    return dateToKey(date);
  },
};
