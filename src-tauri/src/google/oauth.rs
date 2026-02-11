use super::{token_store, GoogleCalendarState, GoogleTokens};
use crate::error::{AppError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use tauri::Emitter;

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const SCOPE: &str = "https://www.googleapis.com/auth/calendar.events";

fn generate_pkce() -> (String, String) {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(&bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let code_challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (code_verifier, code_challenge)
}

pub async fn start_auth_flow(
    google_state: &GoogleCalendarState,
    app_handle: tauri::AppHandle,
) -> Result<String> {
    let (code_verifier, code_challenge) = generate_pkce();
    let state_param = uuid::Uuid::new_v4().to_string();

    // Bind to localhost with OS-assigned port
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::GoogleAuth(format!("Failed to bind listener: {}", e)))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::GoogleAuth(format!("Failed to get port: {}", e)))?
        .port();

    let redirect_uri = format!("http://127.0.0.1:{}", port);

    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&access_type=offline&prompt=consent&state={}",
        AUTH_URL,
        url::form_urlencoded::byte_serialize(google_state.client_id.as_bytes()).collect::<String>(),
        url::form_urlencoded::byte_serialize(redirect_uri.as_bytes()).collect::<String>(),
        url::form_urlencoded::byte_serialize(SCOPE.as_bytes()).collect::<String>(),
        code_challenge,
        state_param,
    );

    let client_id = google_state.client_id.clone();
    let client_secret = google_state.client_secret.clone();
    let state_param_clone = state_param.clone();

    // Spawn a background task to listen for the callback
    let google_state = google_state.clone();
    tauri::async_runtime::spawn(async move {
        // Set a timeout of 5 minutes
        let listener_result = tokio::time::timeout(
            std::time::Duration::from_secs(300),
            tokio::task::spawn_blocking(move || {
                handle_callback(
                    listener,
                    &state_param_clone,
                    &code_verifier,
                    &redirect_uri,
                    &client_id,
                    &client_secret,
                )
            }),
        )
        .await;

        match listener_result {
            Ok(Ok(Ok(tokens))) => {
                token_store::store_tokens(&tokens);
                *google_state.tokens.lock().await = Some(tokens);
                let _ = app_handle.emit("google-auth-complete", true);
            }
            Ok(Ok(Err(e))) => {
                eprintln!("OAuth callback error: {}", e);
                let _ = app_handle.emit("google-auth-complete", false);
            }
            Ok(Err(e)) => {
                eprintln!("OAuth task error: {}", e);
                let _ = app_handle.emit("google-auth-complete", false);
            }
            Err(_) => {
                eprintln!("OAuth callback timed out");
                let _ = app_handle.emit("google-auth-complete", false);
            }
        }
    });

    Ok(auth_url)
}

fn handle_callback(
    listener: TcpListener,
    expected_state: &str,
    code_verifier: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<GoogleTokens> {
    // Accept exactly one connection
    let (mut stream, _) = listener
        .accept()
        .map_err(|e| AppError::GoogleAuth(format!("Failed to accept connection: {}", e)))?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| AppError::GoogleAuth(format!("Failed to read request: {}", e)))?;

    // Parse the URL from GET request
    let url_part = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AppError::GoogleAuth("Invalid HTTP request".to_string()))?;

    let full_url = format!("http://localhost{}", url_part);
    let parsed = url::Url::parse(&full_url)
        .map_err(|e| AppError::GoogleAuth(format!("Failed to parse URL: {}", e)))?;

    // Extract query parameters
    let params: std::collections::HashMap<String, String> =
        parsed.query_pairs().into_owned().collect();

    // Validate state
    let received_state = params
        .get("state")
        .ok_or_else(|| AppError::GoogleAuth("Missing state parameter".to_string()))?;
    if received_state != expected_state {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authentication failed</h1><p>Invalid state parameter.</p></body></html>";
        let _ = stream.write_all(response.as_bytes());
        return Err(AppError::GoogleAuth("State mismatch".to_string()));
    }

    // Check for error
    if let Some(error) = params.get("error") {
        let response = format!("HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authentication failed</h1><p>{}</p></body></html>", error);
        let _ = stream.write_all(response.as_bytes());
        return Err(AppError::GoogleAuth(format!("Auth error: {}", error)));
    }

    let code = params
        .get("code")
        .ok_or_else(|| AppError::GoogleAuth("Missing authorization code".to_string()))?;

    // Exchange code for tokens (blocking HTTP call)
    let tokens =
        exchange_code_blocking(code, code_verifier, redirect_uri, client_id, client_secret)?;

    // Send success response
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body style=\"font-family: system-ui; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #f5f5f5;\"><div style=\"text-align: center; padding: 2rem; background: white; border-radius: 12px; box-shadow: 0 2px 10px rgba(0,0,0,0.1);\"><h1 style=\"color: #22c55e;\">Authentication successful!</h1><p>You can close this window and return to MyTodos.</p></div></body></html>";
    let _ = stream.write_all(response.as_bytes());

    Ok(tokens)
}

fn exchange_code_blocking(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<GoogleTokens> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
            ("code_verifier", code_verifier),
        ])
        .send()
        .map_err(|e| AppError::GoogleAuth(format!("Token exchange failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(AppError::GoogleAuth(format!(
            "Token exchange returned {}: {}",
            status, body
        )));
    }

    let token_response: serde_json::Value = response
        .json()
        .map_err(|e| AppError::GoogleAuth(format!("Failed to parse token response: {}", e)))?;

    let access_token = token_response["access_token"]
        .as_str()
        .ok_or_else(|| AppError::GoogleAuth("Missing access_token".to_string()))?
        .to_string();
    let refresh_token = token_response["refresh_token"]
        .as_str()
        .ok_or_else(|| AppError::GoogleAuth("Missing refresh_token".to_string()))?
        .to_string();
    let expires_in = token_response["expires_in"].as_i64().unwrap_or(3600);
    let expires_at = chrono::Utc::now().timestamp() + expires_in;

    Ok(GoogleTokens {
        access_token,
        refresh_token,
        expires_at,
    })
}

pub async fn refresh_token_if_needed(google_state: &GoogleCalendarState) -> Result<String> {
    let tokens_guard = google_state.tokens.lock().await;
    let tokens = tokens_guard
        .as_ref()
        .ok_or_else(|| AppError::GoogleAuth("Not connected to Google".to_string()))?;

    let now = chrono::Utc::now().timestamp();
    // Refresh if token expires within 60 seconds
    if tokens.expires_at > now + 60 {
        return Ok(tokens.access_token.clone());
    }

    let refresh_token = tokens.refresh_token.clone();
    let client_id = google_state.client_id.clone();
    let client_secret = google_state.client_secret.clone();

    // Need to drop the lock before making the HTTP call
    drop(tokens_guard);

    let client = reqwest::Client::new();
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| AppError::GoogleAuth(format!("Token refresh failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        // If refresh fails with 400/401, tokens are invalid
        if status.as_u16() == 400 || status.as_u16() == 401 {
            token_store::clear_tokens();
            let mut tokens_guard = google_state.tokens.lock().await;
            *tokens_guard = None;
        }
        return Err(AppError::GoogleAuth(format!(
            "Token refresh returned {}: {}",
            status, body
        )));
    }

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::GoogleAuth(format!("Failed to parse refresh response: {}", e)))?;

    let new_access_token = token_response["access_token"]
        .as_str()
        .ok_or_else(|| AppError::GoogleAuth("Missing access_token in refresh".to_string()))?
        .to_string();
    let expires_in = token_response["expires_in"].as_i64().unwrap_or(3600);
    let new_expires_at = chrono::Utc::now().timestamp() + expires_in;

    let new_tokens = GoogleTokens {
        access_token: new_access_token.clone(),
        refresh_token: token_response["refresh_token"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or(refresh_token),
        expires_at: new_expires_at,
    };

    token_store::store_tokens(&new_tokens);
    let mut tokens_guard = google_state.tokens.lock().await;
    *tokens_guard = Some(new_tokens);

    Ok(new_access_token)
}
