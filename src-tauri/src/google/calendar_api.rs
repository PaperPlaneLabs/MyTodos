use crate::error::{AppError, Result};
use serde_json::json;

const BASE_URL: &str = "https://www.googleapis.com/calendar/v3";

pub async fn create_event(
    access_token: &str,
    title: &str,
    description: Option<&str>,
    date: &str,
) -> Result<String> {
    let client = reqwest::Client::new();

    let mut body = json!({
        "summary": title,
        "start": { "date": date },
        "end": { "date": date },
        "transparency": "transparent"
    });

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
) -> Result<()> {
    let client = reqwest::Client::new();

    let mut body = json!({
        "summary": title,
        "start": { "date": date },
        "end": { "date": date },
        "transparency": "transparent"
    });

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
