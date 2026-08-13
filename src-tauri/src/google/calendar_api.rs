use crate::db::models::GoogleCalendarEvent;
use crate::error::{AppError, Result};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime, TimeZone};
use serde_json::json;

const BASE_URL: &str = "https://www.googleapis.com/calendar/v3";

pub async fn create_event(
    access_token: &str,
    title: &str,
    description: Option<&str>,
    date: &str,
    planned_duration_minutes: Option<i64>,
) -> Result<String> {
    let client = reqwest::Client::new();

    let mut body = task_event_body(title, date, planned_duration_minutes)?;

    if let Some(desc) = description {
        body["description"] = json!(desc);
    }

    let response = client
        .post(format!("{}/calendars/primary/events", BASE_URL))
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::GoogleApi(format!("Failed to create event: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::GoogleApi(format!(
            "Create event returned {}: {}",
            status, body
        )));
    }

    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::GoogleApi(format!("Failed to parse event response: {}", e)))?;

    let event_id = result["id"]
        .as_str()
        .ok_or_else(|| AppError::GoogleApi("Missing event id in response".to_string()))?
        .to_string();

    Ok(event_id)
}

pub async fn update_event(
    access_token: &str,
    event_id: &str,
    title: &str,
    description: Option<&str>,
    date: &str,
    planned_duration_minutes: Option<i64>,
) -> Result<()> {
    let client = reqwest::Client::new();

    let mut body = task_event_body(title, date, planned_duration_minutes)?;

    if let Some(desc) = description {
        body["description"] = json!(desc);
    }

    let response = client
        .patch(format!(
            "{}/calendars/primary/events/{}",
            BASE_URL, event_id
        ))
        .bearer_auth(access_token)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::GoogleApi(format!("Failed to update event: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::GoogleApi(format!(
            "Update event returned {}: {}",
            status, body
        )));
    }

    Ok(())
}

fn task_event_body(
    title: &str,
    deadline: &str,
    planned_duration_minutes: Option<i64>,
) -> Result<serde_json::Value> {
    if deadline.contains('T') {
        let parsed = NaiveDateTime::parse_from_str(deadline, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(deadline, "%Y-%m-%dT%H:%M"))
            .map_err(|_| AppError::InvalidInput("Invalid timed task deadline".into()))?;
        let start = Local
            .from_local_datetime(&parsed)
            .earliest()
            .ok_or_else(|| {
                AppError::InvalidInput("Task deadline is not a valid local time".into())
            })?;
        let end = start + Duration::minutes(planned_duration_minutes.unwrap_or(30).max(5));
        Ok(json!({
            "summary": title,
            "start": { "dateTime": start.to_rfc3339() },
            "end": { "dateTime": end.to_rfc3339() },
            "transparency": "transparent"
        }))
    } else {
        let start = NaiveDate::parse_from_str(deadline, "%Y-%m-%d")
            .map_err(|_| AppError::InvalidInput("Invalid task deadline date".into()))?;
        Ok(json!({
            "summary": title,
            "start": { "date": start.format("%Y-%m-%d").to_string() },
            "end": { "date": (start + Duration::days(1)).format("%Y-%m-%d").to_string() },
            "transparency": "transparent"
        }))
    }
}

pub async fn list_events(
    access_token: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<GoogleCalendarEvent>> {
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput("Invalid Google range start".into()))?;
    let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| AppError::InvalidInput("Invalid Google range end".into()))?
        + Duration::days(1);
    let start_local = Local
        .from_local_datetime(&start.and_hms_opt(0, 0, 0).expect("midnight exists"))
        .earliest()
        .ok_or_else(|| AppError::InvalidInput("Invalid local range start".into()))?;
    let end_local = Local
        .from_local_datetime(&end.and_hms_opt(0, 0, 0).expect("midnight exists"))
        .earliest()
        .ok_or_else(|| AppError::InvalidInput("Invalid local range end".into()))?;

    let client = reqwest::Client::new();
    let mut page_token: Option<String> = None;
    let mut events = Vec::new();
    loop {
        let mut query = vec![
            ("timeMin", start_local.to_rfc3339()),
            ("timeMax", end_local.to_rfc3339()),
            ("singleEvents", "true".into()),
            ("showDeleted", "false".into()),
            ("maxResults", "2500".into()),
        ];
        if let Some(token) = page_token.as_ref() {
            query.push(("pageToken", token.clone()));
        }
        let response = client
            .get(format!("{}/calendars/primary/events", BASE_URL))
            .bearer_auth(access_token)
            .query(&query)
            .send()
            .await
            .map_err(|error| AppError::GoogleApi(format!("Failed to list events: {}", error)))?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::GoogleApi(format!(
                "List events returned {}: {}",
                status, body
            )));
        }
        let payload: serde_json::Value = response
            .json()
            .await
            .map_err(|error| AppError::GoogleApi(format!("Failed to parse events: {}", error)))?;
        if let Some(items) = payload["items"].as_array() {
            for item in items {
                if let Some(event) = parse_google_event(item) {
                    events.push(event);
                }
            }
        }
        page_token = payload["nextPageToken"].as_str().map(ToOwned::to_owned);
        if page_token.is_none() {
            break;
        }
    }
    Ok(events)
}

fn parse_google_event(value: &serde_json::Value) -> Option<GoogleCalendarEvent> {
    let external_id = value["id"].as_str()?.to_string();
    let title = value["summary"]
        .as_str()
        .unwrap_or("Untitled event")
        .to_string();
    let description = value["description"].as_str().map(ToOwned::to_owned);
    let html_link = value["htmlLink"].as_str().map(ToOwned::to_owned);
    if let Some(start_date) = value["start"]["date"].as_str() {
        let end_date = value["end"]["date"].as_str().unwrap_or(start_date);
        return Some(GoogleCalendarEvent {
            external_id,
            title,
            description,
            is_all_day: true,
            start_date: start_date.into(),
            end_date: end_date.into(),
            start_at: None,
            end_at: None,
            timezone: value["start"]["timeZone"].as_str().map(ToOwned::to_owned),
            html_link,
            color: "#4285f4".into(),
        });
    }
    let start_raw = value["start"]["dateTime"].as_str()?;
    let end_raw = value["end"]["dateTime"].as_str()?;
    let start = chrono::DateTime::parse_from_rfc3339(start_raw).ok()?;
    let end = chrono::DateTime::parse_from_rfc3339(end_raw).ok()?;
    Some(GoogleCalendarEvent {
        external_id,
        title,
        description,
        is_all_day: false,
        start_date: start
            .with_timezone(&Local)
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
        end_date: (end.with_timezone(&Local).date_naive() + Duration::days(1))
            .format("%Y-%m-%d")
            .to_string(),
        start_at: Some(start.timestamp()),
        end_at: Some(end.timestamp()),
        timezone: value["start"]["timeZone"].as_str().map(ToOwned::to_owned),
        html_link,
        color: "#4285f4".into(),
    })
}

pub async fn delete_event(access_token: &str, event_id: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .delete(format!(
            "{}/calendars/primary/events/{}",
            BASE_URL, event_id
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::GoogleApi(format!("Failed to delete event: {}", e)))?;

    // 404 means event already deleted - that's fine
    if response.status().as_u16() == 404 {
        return Ok(());
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::GoogleApi(format!(
            "Delete event returned {}: {}",
            status, body
        )));
    }

    Ok(())
}
