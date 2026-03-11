#![cfg(feature = "server")]
//! Server-only infrastructure shared across all endpoint modules.

use std::sync::OnceLock;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use dioxus::prelude::ServerFnError;
use ::spotify::SpotifyClient;

pub mod meetings;
pub mod members;
pub mod reviews;
pub mod spotify;

// Re-export the impl fns so lib.rs can reach them via `server::*`.
pub use meetings::{
    admin_delete_history_entry_impl, admin_reorder_members_impl, admin_set_current_impl,
    admin_update_current_impl, get_current_impl, get_history_impl,
};
pub use members::admin_set_member_password_impl;
pub use reviews::{get_reviews_impl, submit_album_review_impl, submit_track_review_impl};
pub use spotify::{admin_spotify_album_search_impl, get_album_tracks_impl};

// Also re-export verify so lib.rs can call it directly for the verify_member endpoint.
pub use self::members::verify_member_password_internal;

// ---------------------------------------------------------------------------
// Extension trait
// ---------------------------------------------------------------------------

/// Converts any `Display` error into a `ServerFnError` via `.server_err()`.
pub trait IntoServerError<T> {
    fn server_err(self) -> Result<T, ServerFnError>;
}

impl<T, E: std::fmt::Display> IntoServerError<T> for Result<T, E> {
    fn server_err(self) -> Result<T, ServerFnError> {
        self.map_err(|e| ServerFnError::new(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Statics
// ---------------------------------------------------------------------------

pub static SPOTIFY_CLIENT: OnceLock<tokio::sync::Mutex<Option<SpotifyClient>>> =
    OnceLock::new();

static DB: tokio::sync::OnceCell<sqlx::SqlitePool> = tokio::sync::OnceCell::const_new();

const ADMIN_TOKEN_ENV: &str = "ADMIN_TOKEN";

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub async fn get_db() -> Result<&'static sqlx::SqlitePool, ServerFnError> {
    DB.get_or_try_init(|| async {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:database.db".to_string());
        tracing::info!("Connecting to database: {db_url}");
        let pool = crate::db::init_pool(&db_url).await;
        match &pool {
            Ok(_) => tracing::info!("Database ready"),
            Err(e) => tracing::error!("Database initialisation failed: {e}"),
        }
        pool
    })
    .await
    .server_err()
}

/// Eagerly initialise the database pool. Call this on startup to surface errors early.
pub async fn init_db() -> Result<(), ServerFnError> {
    get_db().await?;
    Ok(())
}

pub fn ensure_admin_token(admin_token: &str) -> Result<(), ServerFnError> {
    let expected_hash = std::env::var(ADMIN_TOKEN_ENV)
        .map_err(|_| ServerFnError::new("ADMIN_TOKEN is not configured on the server"))?;

    let parsed_hash = PasswordHash::new(&expected_hash)
        .map_err(|_| ServerFnError::new("ADMIN_TOKEN must be a valid Argon2 PHC hash"))?;

    Argon2::default()
        .verify_password(admin_token.as_bytes(), &parsed_hash)
        .map_err(|_| ServerFnError::new("Unauthorized"))?;

    Ok(())
}

/// Acquire the Spotify client, lazily initialising it from environment variables.
pub async fn get_spotify_client(
) -> Result<tokio::sync::MutexGuard<'static, Option<SpotifyClient>>, ServerFnError> {
    let mutex = SPOTIFY_CLIENT.get_or_init(|| tokio::sync::Mutex::new(None));
    let mut guard = mutex.lock().await;

    if guard.is_none() {
        *guard = Some(SpotifyClient::from_env().server_err()?);
    }

    Ok(guard)
}
