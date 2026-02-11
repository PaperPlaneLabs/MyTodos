pub mod calendar_api;
pub mod oauth;
pub mod sync;
pub mod token_store;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

pub struct GoogleCalendarStateInner {
    pub tokens: Mutex<Option<GoogleTokens>>,
    pub client_id: String,
    pub client_secret: String,
}

/// Arc-wrapped state so it can be cloned into async tasks
pub type GoogleCalendarState = Arc<GoogleCalendarStateInner>;

pub fn create_google_state() -> GoogleCalendarState {
    let client_id = option_env!("GOOGLE_CLIENT_ID").unwrap_or("").to_string();
    let client_secret = option_env!("GOOGLE_CLIENT_SECRET")
        .unwrap_or("")
        .to_string();

    let tokens = token_store::load_tokens();

    Arc::new(GoogleCalendarStateInner {
        tokens: Mutex::new(tokens),
        client_id,
        client_secret,
    })
}

impl GoogleCalendarStateInner {
    pub async fn is_connected(&self) -> bool {
        self.tokens.lock().await.is_some()
    }
}
