//! Calendar invite sending via the Gmail API.

use std::time::Instant;

use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Europe::Stockholm;
use dioxus::prelude::ServerFnError;
use rust_gmail::{CalendarEvent, GmailClient, GmailClientBuilder};
use sqlx::Row;

use crate::api_models::SendInvitesResult;

use super::{ensure_admin_token, get_db, IntoServerError};

const MEETING_LENGTH_HOURS: i64 = 2;

/// Google access tokens last an hour; refresh a bit early.
const CLIENT_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(50 * 60);

struct GmailState {
    builder: GmailClientBuilder,
    client: GmailClient,
    built_at: Instant,
}

static GMAIL: tokio::sync::OnceCell<tokio::sync::Mutex<GmailState>> =
    tokio::sync::OnceCell::const_new();

async fn init_state() -> Result<tokio::sync::Mutex<GmailState>, ServerFnError> {
    let service_account = std::env::var("GMAIL_SERVICE_ACCOUNT_PATH")
        .map_err(|_| ServerFnError::new("GMAIL_SERVICE_ACCOUNT_PATH is not configured"))?;
    let send_from = std::env::var("GMAIL_SEND_FROM")
        .map_err(|_| ServerFnError::new("GMAIL_SEND_FROM is not configured"))?;
    let mock = match std::env::var("GMAIL_MOCK_MODE") {
        Err(_) => false,
        Ok(v) => v
            .trim()
            .parse::<bool>()
            .map_err(|_| ServerFnError::new("GMAIL_MOCK_MODE must be true or false"))?,
    };

    tracing::info!("Loading Gmail service account from {service_account} (mock mode: {mock})");
    let builder = GmailClient::builder(service_account, send_from)
        .server_err()?
        .mock_mode(mock);
    // Requests an access token, which validates the credentials.
    let client = builder.clone().build().await.server_err()?;

    Ok(tokio::sync::Mutex::new(GmailState {
        builder,
        client,
        built_at: Instant::now(),
    }))
}

/// Load and validate the Gmail service account and create the client.
/// Call this on startup to surface configuration errors early.
pub async fn init_gmail() -> Result<(), ServerFnError> {
    GMAIL.get_or_try_init(init_state).await?;
    tracing::info!("Gmail client ready");
    Ok(())
}

/// Get the shared Gmail client, rebuilding it first if its access token is about to expire.
async fn get_gmail_client() -> Result<GmailClient, ServerFnError> {
    let mut state = GMAIL.get_or_try_init(init_state).await?.lock().await;
    if state.built_at.elapsed() > CLIENT_MAX_AGE {
        tracing::debug!("Refreshing Gmail client");
        state.client = state.builder.clone().build().await.server_err()?;
        state.built_at = Instant::now();
    }
    Ok(state.client.clone())
}

pub async fn admin_send_calendar_invites_impl(
    admin_token: String,
) -> Result<SendInvitesResult, ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::info!("POST /api/admin/send-invites");

    let pool = get_db().await?;

    let meeting = sqlx::query(
        "SELECT id, album_name, album_artist, album_spotify_url, picker,
                meeting_date, meeting_time, meeting_location
         FROM meetings WHERE is_current = 1",
    )
    .fetch_optional(pool)
    .await
    .server_err()?
    .ok_or_else(|| ServerFnError::new("Inget nuvarande möte finns"))?;

    let id: String = meeting.get("id");
    let album_name: String = meeting.get("album_name");
    let album_artist: String = meeting.get("album_artist");
    let spotify_url: String = meeting.get("album_spotify_url");
    let picker: String = meeting.get("picker");
    let date: String = meeting.get("meeting_date");
    let time: Option<String> = meeting.get("meeting_time");
    let location: Option<String> = meeting.get("meeting_location");

    let time = time
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| ServerFnError::new("Mötet saknar tid – ange en tid först"))?;

    // Meeting times are entered in Swedish local time.
    let naive = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .ok()
        .zip(NaiveTime::parse_from_str(&time, "%H:%M").ok())
        .map(|(d, t)| d.and_time(t))
        .ok_or_else(|| ServerFnError::new("Ogiltigt datum eller tid för mötet"))?;
    let start = Stockholm
        .from_local_datetime(&naive)
        .earliest()
        .ok_or_else(|| ServerFnError::new("Ogiltig lokal tid för mötet"))?
        .with_timezone(&Utc);
    let end = start + Duration::hours(MEETING_LENGTH_HOURS);

    let mut event = CalendarEvent::new("Albumklubben", start, end)
        .description(format!(
            "{album_name} – {album_artist}\nVald av {picker}\n{spotify_url}"
        ))
        .uid(format!("{id}@albumklubben"));
    if let Some(location) = location.filter(|l| !l.trim().is_empty()) {
        event = event.location(location);
    }

    let recipients: Vec<(String, String)> = sqlx::query(
        "SELECT name, email FROM members
         WHERE deleted_at IS NULL AND email IS NOT NULL AND email != ''
         ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await
    .server_err()?
    .into_iter()
    .map(|r| (r.get("name"), r.get("email")))
    .collect();

    if recipients.is_empty() {
        return Err(ServerFnError::new("Ingen medlem har angett en e-postadress"));
    }

    let client = get_gmail_client().await?;
    let subject = format!("Albumklubben: {album_name} – {album_artist}");

    let mut result = SendInvitesResult {
        sent: Vec::new(),
        failed: Vec::new(),
    };
    for (name, email) in recipients {
        let body = format!(
            "Hej {name}!\n\nNästa Albumklubben-möte är {date} kl. {time}.\n\
             Album: {album_name} – {album_artist} (vald av {picker})\n{spotify_url}\n"
        );
        match client
            .send_calendar_event(&email, &subject, &body, &event)
            .await
        {
            Ok(()) => result.sent.push(name),
            Err(e) => {
                tracing::error!("Failed to send invite to \"{name}\": {e}");
                result.failed.push((name, e.to_string()));
            }
        }
    }

    tracing::info!(
        "POST /api/admin/send-invites → {} sent, {} failed",
        result.sent.len(),
        result.failed.len()
    );
    Ok(result)
}
