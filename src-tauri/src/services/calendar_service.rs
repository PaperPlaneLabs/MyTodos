use crate::db::models::CalendarEvent;
use crate::error::{AppError, Result};
use chrono::{Datelike, Duration, Months, NaiveDate};
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecurrenceRule {
    pub frequency: RecurrenceFrequency,
    #[serde(default = "default_interval")]
    pub interval: u32,
    #[serde(default)]
    pub weekdays: Vec<u32>,
    #[serde(default)]
    pub until: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
}

fn default_interval() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarEventInput {
    pub title: String,
    pub description: Option<String>,
    pub is_all_day: bool,
    pub color: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub start_at: Option<i64>,
    pub end_at: Option<i64>,
    pub timezone: Option<String>,
    pub recurrence_rule: Option<String>,
}

#[derive(Debug, Clone)]
struct StoredCalendarEvent {
    id: i64,
    title: String,
    description: Option<String>,
    is_all_day: bool,
    color: String,
    start_date: String,
    end_date: String,
    start_time: Option<String>,
    end_time: Option<String>,
    start_at: Option<i64>,
    end_at: Option<i64>,
    timezone: Option<String>,
    recurrence_rule: Option<String>,
}

fn parse_date(value: &str, field: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput(format!("{} must use YYYY-MM-DD", field)))
}

fn validate_input(input: &CalendarEventInput) -> Result<()> {
    if input.title.trim().is_empty() {
        return Err(AppError::InvalidInput("Event title cannot be empty".into()));
    }
    let start = parse_date(&input.start_date, "start_date")?;
    let end = parse_date(&input.end_date, "end_date")?;
    if end <= start {
        return Err(AppError::InvalidInput(
            "end_date must be after start_date (end-exclusive)".into(),
        ));
    }
    if !input.is_all_day {
        if input.start_time.is_none() || input.end_time.is_none() {
            return Err(AppError::InvalidInput(
                "Timed events require start_time and end_time".into(),
            ));
        }
        if let (Some(start_at), Some(end_at)) = (input.start_at, input.end_at) {
            if end_at <= start_at {
                return Err(AppError::InvalidInput(
                    "Timed event end must be after its start".into(),
                ));
            }
        }
    }
    if let Some(raw) = input.recurrence_rule.as_deref() {
        let rule: RecurrenceRule = serde_json::from_str(raw)
            .map_err(|_| AppError::InvalidInput("Invalid recurrence rule".into()))?;
        validate_rule(&rule)?;
    }
    Ok(())
}

fn validate_rule(rule: &RecurrenceRule) -> Result<()> {
    if rule.interval == 0 || rule.interval > 999 {
        return Err(AppError::InvalidInput(
            "Recurrence interval must be between 1 and 999".into(),
        ));
    }
    if rule.weekdays.iter().any(|day| *day > 6) {
        return Err(AppError::InvalidInput(
            "Recurrence weekdays must be between 0 and 6".into(),
        ));
    }
    if matches!(rule.count, Some(0)) {
        return Err(AppError::InvalidInput(
            "Recurrence count must be positive".into(),
        ));
    }
    if let Some(until) = rule.until.as_deref() {
        parse_date(until, "recurrence until")?;
    }
    Ok(())
}

fn stored_from_row(row: &Row<'_>) -> rusqlite::Result<StoredCalendarEvent> {
    Ok(StoredCalendarEvent {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        is_all_day: row.get("is_all_day")?,
        color: row
            .get::<_, Option<String>>("color")?
            .unwrap_or_else(|| "#6366f1".into()),
        start_date: row.get("start_date")?,
        end_date: row.get("end_date")?,
        start_time: row.get("start_time")?,
        end_time: row.get("end_time")?,
        start_at: row.get("start_at")?,
        end_at: row.get("end_at")?,
        timezone: row.get("timezone")?,
        recurrence_rule: row.get("recurrence_rule")?,
    })
}

fn calendar_select() -> &'static str {
    "SELECT id, title, description, is_all_day, color, start_date, end_date,
            start_time, end_time, start_at, end_at, timezone, recurrence_rule
     FROM calendar_events"
}

pub fn create_event(conn: &Connection, input: CalendarEventInput) -> Result<CalendarEvent> {
    validate_input(&input)?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO calendar_events
         (title, description, date, is_all_day, color, created_at, start_date,
          end_date, start_time, end_time, start_at, end_at, timezone,
          recurrence_rule, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?3, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?6)",
        params![
            input.title.trim(),
            input.description,
            input.start_date,
            input.is_all_day,
            input.color.unwrap_or_else(|| "#6366f1".into()),
            now,
            input.end_date,
            input.start_time,
            input.end_time,
            input.start_at,
            input.end_at,
            input.timezone,
            input.recurrence_rule,
        ],
    )?;
    get_series(conn, conn.last_insert_rowid())
}

pub fn update_event(
    conn: &Connection,
    event_id: i64,
    input: CalendarEventInput,
) -> Result<CalendarEvent> {
    validate_input(&input)?;
    let rows = conn.execute(
        "UPDATE calendar_events
         SET title = ?1, description = ?2, date = ?3, is_all_day = ?4,
             color = ?5, start_date = ?3, end_date = ?6, start_time = ?7,
             end_time = ?8, start_at = ?9, end_at = ?10, timezone = ?11,
             recurrence_rule = ?12, updated_at = ?13
         WHERE id = ?14",
        params![
            input.title.trim(),
            input.description,
            input.start_date,
            input.is_all_day,
            input.color.unwrap_or_else(|| "#6366f1".into()),
            input.end_date,
            input.start_time,
            input.end_time,
            input.start_at,
            input.end_at,
            input.timezone,
            input.recurrence_rule,
            chrono::Utc::now().timestamp(),
            event_id,
        ],
    )?;
    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "Calendar event {} not found",
            event_id
        )));
    }
    get_series(conn, event_id)
}

pub fn delete_event(conn: &Connection, event_id: i64) -> Result<()> {
    if conn.execute("DELETE FROM calendar_events WHERE id = ?1", [event_id])? == 0 {
        return Err(AppError::NotFound(format!(
            "Calendar event {} not found",
            event_id
        )));
    }
    Ok(())
}

fn get_stored(conn: &Connection, event_id: i64) -> Result<StoredCalendarEvent> {
    conn.query_row(
        &format!("{} WHERE id = ?1", calendar_select()),
        [event_id],
        stored_from_row,
    )
    .map_err(|_| AppError::NotFound(format!("Calendar event {} not found", event_id)))
}

fn get_series(conn: &Connection, event_id: i64) -> Result<CalendarEvent> {
    let stored = get_stored(conn, event_id)?;
    Ok(to_occurrence(&stored, &stored.start_date, 0))
}

fn to_occurrence(
    stored: &StoredCalendarEvent,
    occurrence_date: &str,
    shift_days: i64,
) -> CalendarEvent {
    let start = NaiveDate::parse_from_str(occurrence_date, "%Y-%m-%d").unwrap_or_else(|_| {
        NaiveDate::parse_from_str(&stored.start_date, "%Y-%m-%d").expect("stored date is valid")
    });
    let original_start =
        NaiveDate::parse_from_str(&stored.start_date, "%Y-%m-%d").expect("stored date is valid");
    let original_end =
        NaiveDate::parse_from_str(&stored.end_date, "%Y-%m-%d").expect("stored date is valid");
    let duration_days = (original_end - original_start).num_days().max(1);
    let end = start + Duration::days(duration_days);
    CalendarEvent {
        id: stored.id,
        series_id: stored.id,
        series_start_date: stored.start_date.clone(),
        series_end_date: stored.end_date.clone(),
        occurrence_key: format!("{}:{}", stored.id, occurrence_date),
        title: stored.title.clone(),
        description: stored.description.clone(),
        date: stored
            .start_time
            .as_ref()
            .map(|time| format!("{}T{}:00", occurrence_date, time))
            .unwrap_or_else(|| occurrence_date.to_string()),
        is_all_day: stored.is_all_day,
        color: stored.color.clone(),
        start_date: occurrence_date.to_string(),
        end_date: end.format("%Y-%m-%d").to_string(),
        start_time: stored.start_time.clone(),
        end_time: stored.end_time.clone(),
        start_at: stored.start_at.map(|value| value + shift_days * 86_400),
        end_at: stored.end_at.map(|value| value + shift_days * 86_400),
        timezone: stored.timezone.clone(),
        recurrence_rule: stored.recurrence_rule.clone(),
    }
}

fn monthly_candidate(anchor: NaiveDate, months: u32) -> NaiveDate {
    let first = anchor.with_day(1).expect("day one exists");
    let target_first = first
        .checked_add_months(Months::new(months))
        .expect("calendar recurrence month is representable");
    let next_month = target_first
        .checked_add_months(Months::new(1))
        .expect("next recurrence month is representable");
    let last_day = (next_month - Duration::days(1)).day();
    target_first
        .with_day(anchor.day().min(last_day))
        .expect("clamped day exists")
}

pub fn recurrence_dates(
    anchor: NaiveDate,
    rule: &RecurrenceRule,
    range_start: NaiveDate,
    range_end: NaiveDate,
) -> Result<Vec<NaiveDate>> {
    validate_rule(rule)?;
    let until = rule
        .until
        .as_deref()
        .map(|value| parse_date(value, "recurrence until"))
        .transpose()?;
    let max_date = until.map_or(range_end, |date| date.min(range_end));
    if max_date < anchor {
        return Ok(Vec::new());
    }

    let mut dates = Vec::new();
    let mut emitted = 0_u32;
    let limit = rule.count.unwrap_or(u32::MAX);
    let push = |date: NaiveDate, dates: &mut Vec<NaiveDate>, emitted: &mut u32| {
        if date >= range_start && date <= range_end {
            dates.push(date);
        }
        *emitted += 1;
    };

    match rule.frequency {
        RecurrenceFrequency::Daily => {
            let mut date = anchor;
            while date <= max_date && emitted < limit {
                push(date, &mut dates, &mut emitted);
                date += Duration::days(rule.interval as i64);
            }
        }
        RecurrenceFrequency::Weekly => {
            let weekdays = if rule.weekdays.is_empty() {
                vec![anchor.weekday().num_days_from_sunday()]
            } else {
                rule.weekdays.clone()
            };
            let mut date = anchor;
            while date <= max_date && emitted < limit {
                let days_from_anchor = (date - anchor).num_days();
                let week_index = days_from_anchor / 7;
                if week_index % rule.interval as i64 == 0
                    && weekdays.contains(&date.weekday().num_days_from_sunday())
                {
                    push(date, &mut dates, &mut emitted);
                }
                date += Duration::days(1);
            }
        }
        RecurrenceFrequency::Monthly => {
            let mut iteration = 0_u32;
            while emitted < limit {
                let date = monthly_candidate(anchor, iteration.saturating_mul(rule.interval));
                if date > max_date {
                    break;
                }
                push(date, &mut dates, &mut emitted);
                iteration += 1;
            }
        }
        RecurrenceFrequency::Yearly => {
            let mut iteration = 0_u32;
            while emitted < limit {
                let date = monthly_candidate(
                    anchor,
                    iteration.saturating_mul(rule.interval).saturating_mul(12),
                );
                if date > max_date {
                    break;
                }
                push(date, &mut dates, &mut emitted);
                iteration += 1;
            }
        }
    }
    Ok(dates)
}

pub fn list_events(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<CalendarEvent>> {
    let range_start = parse_date(start_date, "start_date")?;
    let range_end = parse_date(end_date, "end_date")?;
    if range_end < range_start {
        return Err(AppError::InvalidInput(
            "Calendar range end must not precede start".into(),
        ));
    }
    let range_end_exclusive = range_end + Duration::days(1);
    let mut stmt = conn.prepare(&format!(
        "{} WHERE recurrence_rule IS NOT NULL
            OR (start_date < ?2 AND end_date > ?1)",
        calendar_select()
    ))?;
    let stored = stmt
        .query_map(
            params![
                start_date,
                range_end_exclusive.format("%Y-%m-%d").to_string()
            ],
            stored_from_row,
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut events = Vec::new();
    for event in stored {
        if let Some(raw) = event.recurrence_rule.as_deref() {
            let rule: RecurrenceRule = serde_json::from_str(raw).map_err(|_| {
                AppError::InvalidInput(format!("Invalid recurrence rule on event {}", event.id))
            })?;
            let anchor = parse_date(&event.start_date, "event start_date")?;
            for date in recurrence_dates(anchor, &rule, range_start, range_end)? {
                events.push(to_occurrence(
                    &event,
                    &date.format("%Y-%m-%d").to_string(),
                    (date - anchor).num_days(),
                ));
            }
        } else {
            events.push(to_occurrence(&event, &event.start_date, 0));
        }
    }
    events.sort_by(|left, right| {
        (&left.start_date, &left.start_time, &left.title).cmp(&(
            &right.start_date,
            &right.start_time,
            &right.title,
        ))
    });
    Ok(events)
}
