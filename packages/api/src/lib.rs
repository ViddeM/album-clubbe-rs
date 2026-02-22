//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
#[cfg(feature = "server")]
use std::sync::OnceLock;

use crate::api_models::{Data, SpotifyAlbumSearchItem};

pub mod api_models;

#[cfg(feature = "server")]
mod db;

#[cfg(feature = "server")]
static SPOTIFY_CLIENT: OnceLock<tokio::sync::Mutex<Option<spotify::SpotifyClient>>> =
    OnceLock::new();

#[cfg(feature = "server")]
static DB: tokio::sync::OnceCell<sqlx::SqlitePool> = tokio::sync::OnceCell::const_new();

#[allow(dead_code)]
const ADMIN_TOKEN_ENV: &str = "ADMIN_TOKEN";

#[cfg(feature = "server")]
async fn get_db() -> Result<&'static sqlx::SqlitePool, ServerFnError> {
    DB.get_or_try_init(|| async {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:database.db".to_string());
        db::init_pool(&db_url).await
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[allow(dead_code)]
fn ensure_admin_token(admin_token: &str) -> Result<(), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        let _ = admin_token;
        return Err(ServerFnError::new(
            "Admin auth is only available on server builds",
        ));
    }

    #[cfg(feature = "server")]
    {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let expected_hash = std::env::var(ADMIN_TOKEN_ENV)
            .map_err(|_| ServerFnError::new("ADMIN_TOKEN is not configured on the server"))?;

        println!("STORED HASH {expected_hash}");

        let parsed_hash = PasswordHash::new(&expected_hash)
            .map_err(|_| ServerFnError::new("ADMIN_TOKEN must be a valid Argon2 PHC hash"))?;

        Argon2::default()
            .verify_password(admin_token.as_bytes(), &parsed_hash)
            .map_err(|_| ServerFnError::new("Unauthorized"))?;

        Ok(())
    }
}

/// Get the current album, next meeting and member list.
#[get("/api/info")]
pub async fn get_current() -> Result<Data, ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Ok(Data {
            current_album: None,
            next_meeting: None,
            current_person: None,
            members: Vec::new(),
        });
    }

    #[cfg(feature = "server")]
    {
        use sqlx::Row;

        let pool = get_db().await?;

        let members: Vec<String> =
            sqlx::query_scalar("SELECT name FROM members ORDER BY sort_order")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;

        let row = sqlx::query(
            "SELECT album_id, album_name, album_artist, album_art_url, album_spotify_url,
                    picker, meeting_date, meeting_time, meeting_location
             FROM meetings WHERE is_current = 1",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        let member_names: Vec<crate::api_models::Name> =
            members.into_iter().map(|m| m.into()).collect();

        match row {
            None => Ok(Data {
                current_album: None,
                next_meeting: None,
                current_person: None,
                members: member_names,
            }),
            Some(row) => {
                let meeting_date: Option<String> = row.get("meeting_date");
                let meeting_time: Option<String> = row.get("meeting_time");
                let meeting_location: Option<String> = row.get("meeting_location");

                let next_meeting = if meeting_date.is_some()
                    || meeting_time.is_some()
                    || meeting_location.is_some()
                {
                    Some(crate::api_models::Meeting {
                        date: meeting_date.unwrap_or_default(),
                        time: meeting_time,
                        location: meeting_location,
                    })
                } else {
                    None
                };

                Ok(Data {
                    current_album: Some(crate::api_models::Album {
                        name: row.get("album_name"),
                        artist: row.get("album_artist"),
                        album_art: row.get("album_art_url"),
                        spotify_url: row.get("album_spotify_url"),
                    }),
                    next_meeting,
                    current_person: Some(row.get::<String, _>("picker").into()),
                    members: member_names,
                })
            }
        }
    }
}

/// Set the current album, meeting info and picker. Archives the previous state to history.
#[post("/api/admin/set-current")]
pub async fn admin_set_current(
    admin_token: String,
    album_id: String,
    album_name: String,
    album_artist: String,
    album_art_url: String,
    album_spotify_url: String,
    picker: String,
    meeting_date: Option<String>,
    meeting_time: Option<String>,
    meeting_location: Option<String>,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;

    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        use uuid::Uuid;

        let pool = get_db().await?;

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Demote the previous current entry to history.
        sqlx::query("UPDATE meetings SET is_current = 0 WHERE is_current = 1")
            .execute(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Insert the new current entry with a fresh UUID.
        sqlx::query(
            "INSERT INTO meetings
                (id, is_current, album_id, album_name, album_artist, album_art_url,
                 album_spotify_url, picker, meeting_date, meeting_time, meeting_location)
             VALUES (?, 1, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(album_id)
        .bind(album_name)
        .bind(album_artist)
        .bind(album_art_url)
        .bind(album_spotify_url)
        .bind(picker)
        .bind(meeting_date)
        .bind(meeting_time)
        .bind(meeting_location)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(())
    }
}

/// Reorder the member list. `ordered_names` must contain every existing member name
/// in the desired display order; each name is assigned a `sort_order` equal to its index.
#[post("/api/admin/reorder-members")]
pub async fn admin_reorder_members(
    admin_token: String,
    ordered_names: Vec<String>,
) -> Result<(), ServerFnError> {
    ensure_admin_token(&admin_token)?;

    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        let pool = get_db().await?;

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        for (i, name) in ordered_names.iter().enumerate() {
            sqlx::query("UPDATE members SET sort_order = ? WHERE name = ?")
                .bind(i as i64)
                .bind(name)
                .execute(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(())
    }
}

#[post("/api/admin/spotify/search")]
pub async fn admin_spotify_album_search(
    admin_token: String,
    query: String,
) -> Result<Vec<SpotifyAlbumSearchItem>, ServerFnError> {
    ensure_admin_token(&admin_token)?;

    let search_term = query.trim();
    if search_term.is_empty() {
        return Ok(Vec::new());
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = search_term;
        return Err(ServerFnError::new(
            "Spotify search is only available on server builds",
        ));
    }

    #[cfg(feature = "server")]
    {
        let spotify_client = SPOTIFY_CLIENT.get_or_init(|| tokio::sync::Mutex::new(None));
        let mut spotify_client = spotify_client.lock().await;

        if spotify_client.is_none() {
            *spotify_client = Some(
                spotify::SpotifyClient::from_env()
                    .map_err(|e| ServerFnError::new(e.to_string()))?,
            );
        }

        let spotify_client = spotify_client
            .as_mut()
            .ok_or_else(|| ServerFnError::new("Failed to initialize Spotify client"))?;

        let albums = spotify_client
            .search_albums(search_term)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .into_iter()
            .map(|album| SpotifyAlbumSearchItem {
                id: album.id,
                name: album.name,
                artists: album.artists,
                image_url: album.image_url,
                spotify_url: album.spotify_url,
            })
            .collect::<Vec<_>>();

        Ok(albums)
    }
}
