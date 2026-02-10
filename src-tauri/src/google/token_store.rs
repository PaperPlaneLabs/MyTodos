use super::GoogleTokens;

const SERVICE_NAME: &str = "com.pintu.my-todos";
const KEY_ACCESS_TOKEN: &str = "google_access_token";
const KEY_REFRESH_TOKEN: &str = "google_refresh_token";
const KEY_EXPIRES_AT: &str = "google_token_expires_at";

pub fn store_tokens(tokens: &GoogleTokens) {
    let _ = keyring::Entry::new(SERVICE_NAME, KEY_ACCESS_TOKEN)
        .and_then(|e| e.set_password(&tokens.access_token));
    let _ = keyring::Entry::new(SERVICE_NAME, KEY_REFRESH_TOKEN)
        .and_then(|e| e.set_password(&tokens.refresh_token));
    let _ = keyring::Entry::new(SERVICE_NAME, KEY_EXPIRES_AT)
        .and_then(|e| e.set_password(&tokens.expires_at.to_string()));
}

pub fn load_tokens() -> Option<GoogleTokens> {
    let access_token = keyring::Entry::new(SERVICE_NAME, KEY_ACCESS_TOKEN)
        .ok()?
        .get_password()
        .ok()?;
    let refresh_token = keyring::Entry::new(SERVICE_NAME, KEY_REFRESH_TOKEN)
        .ok()?
        .get_password()
        .ok()?;
    let expires_at: i64 = keyring::Entry::new(SERVICE_NAME, KEY_EXPIRES_AT)
        .ok()?
        .get_password()
        .ok()?
        .parse()
        .ok()?;

    Some(GoogleTokens {
        access_token,
        refresh_token,
        expires_at,
    })
}

pub fn clear_tokens() {
    let _ = keyring::Entry::new(SERVICE_NAME, KEY_ACCESS_TOKEN)
        .and_then(|e| e.delete_credential());
    let _ = keyring::Entry::new(SERVICE_NAME, KEY_REFRESH_TOKEN)
        .and_then(|e| e.delete_credential());
    let _ = keyring::Entry::new(SERVICE_NAME, KEY_EXPIRES_AT)
        .and_then(|e| e.delete_credential());
}
