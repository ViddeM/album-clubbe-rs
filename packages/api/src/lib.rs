//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
#[cfg(feature = "server")]
use std::sync::OnceLock;

#[cfg(feature = "server")]
use crate::api_models::{AlbumReview, TrackReview};
use crate::api_models::{AlbumTrack, Data, HistoryEntry, Reviews, SetCurrentRequest, SpotifyAlbumSearchItem};

pub mod api_models;

#[cfg(feature = "server")]
mod db;

#[cfg(feature = "server")]
static SPOTIFY_CLIENT: OnceLock<tokio::sync::Mutex<Option<spotify::SpotifyClient>>> =
    OnceLock::new();

#[cfg(feature = "server")]
static DB: tokio::sync::OnceCell<sqlx::SqlitePool> = tokio::sync::OnceCell::const_new();

#[cfg(feature = "server")]
const ADMIN_TOKEN_ENV: &str = "ADMIN_TOKEN";

#[cfg(feature = "server")]
async fn get_db() -> Result<&'static sqlx::SqlitePool, ServerFnError> {
    DB.get_or_try_init(|| async {
        let db_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:database.db".to_string());
        tracing::info!("Connecting to database: {db_url}");
        let pool = db::init_pool(&db_url).await;
        match &pool {
            Ok(_) => tracing::info!("Database ready"),
            Err(e) => tracing::error!("Database initialisation failed: {e}"),
        }
        pool
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Eagerly initialise the database pool. Call this on startup to surface errors early.
#[cfg(feature = "server")]
pub async fn init_db() -> Result<(), ServerFnError> {
    get_db().await?;
    Ok(())
}

#[cfg(feature = "server")]
fn ensure_admin_token(admin_token: &str) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let expected_hash = std::env::var(ADMIN_TOKEN_ENV)
        .map_err(|_| ServerFnError::new("ADMIN_TOKEN is not configured on the server"))?;

    let parsed_hash = PasswordHash::new(&expected_hash)
        .map_err(|_| ServerFnError::new("ADMIN_TOKEN must be a valid Argon2 PHC hash"))?;

    Argon2::default()
        .verify_password(admin_token.as_bytes(), &parsed_hash)
        .map_err(|_| ServerFnError::new("Unauthorized"))?;

    Ok(())
}

/// Get the current album, next meeting and member list.
#[get("/api/info")]
pub async fn get_current() -> Result<Data, ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        tracing::debug!("GET /api/info");
        use sqlx::Row;

        let pool = get_db().await?;

        let members: Vec<String> =
            sqlx::query_scalar("SELECT name FROM members ORDER BY sort_order")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;

        let row = sqlx::query(
            "SELECT id, album_id, album_name, album_artist, album_art_url, album_spotify_url,
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
                current_meeting_id: None,
                current_album: None,
                next_meeting: None,
                current_person: None,
                members: member_names,
            }),
            Some(row) => {
                let meeting_date: String = row.get("meeting_date");
                let meeting_time: Option<String> = row.get("meeting_time");
                let meeting_location: Option<String> = row.get("meeting_location");

                let next_meeting = Some(crate::api_models::Meeting {
                    date: meeting_date,
                    time: meeting_time,
                    location: meeting_location,
                });

                Ok(Data {
                    current_meeting_id: Some(row.get("id")),
                    current_album: Some(crate::api_models::Album {
                        id: row.get("album_id"),
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

/// Get all past (non-current) meetings, ordered by meeting date ascending.
#[get("/api/history")]
pub async fn get_history() -> Result<Vec<HistoryEntry>, ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        tracing::debug!("GET /api/history");
        use sqlx::Row;

        let pool = get_db().await?;

        let rows = sqlx::query(
            "SELECT id, album_name, album_artist, album_art_url, album_spotify_url, picker, recorded_at,
                    meeting_date, meeting_time, meeting_location
             FROM meetings
             WHERE is_current = 0
             ORDER BY meeting_date ASC",
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        tracing::debug!("GET /api/history → {} entries", rows.len());
        Ok(rows
            .into_iter()
            .map(|row| HistoryEntry {
                id: row.get("id"),
                album_name: row.get("album_name"),
                album_artist: row.get("album_artist"),
                album_art: row.get("album_art_url"),
                spotify_url: row.get("album_spotify_url"),
                picker: row.get("picker"),
                recorded_at: row.get("recorded_at"),
                meeting_date: row.get("meeting_date"),
                meeting_time: row.get("meeting_time"),
                meeting_location: row.get("meeting_location"),
            })
            .collect())
    }
}

/// Set the current album, meeting info and picker. Archives the previous state to history.
#[post("/api/admin/set-current")]
pub async fn admin_set_current(
    admin_token: String,
    req: SetCurrentRequest,
) -> Result<(), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        ensure_admin_token(&admin_token)?;
        use uuid::Uuid;

        tracing::info!("POST /api/admin/set-current album=\"{}\" picker=\"{}\" date=\"{}\"", req.album_name, req.picker, req.meeting_date);
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
        .bind(req.album_id)
        .bind(req.album_name)
        .bind(req.album_artist)
        .bind(req.album_art_url)
        .bind(req.album_spotify_url)
        .bind(req.picker)
        .bind(req.meeting_date)
        .bind(req.meeting_time)
        .bind(req.meeting_location)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        tracing::info!("POST /api/admin/set-current → ok");
        Ok(())
    }
}

/// Update the current meeting / album in-place (without archiving to history).
#[post("/api/admin/update-current")]
pub async fn admin_update_current(
    admin_token: String,
    req: SetCurrentRequest,
) -> Result<(), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        ensure_admin_token(&admin_token)?;
        tracing::info!("POST /api/admin/update-current album=\"{}\" picker=\"{}\" date=\"{}\"", req.album_name, req.picker, req.meeting_date);
        let pool = get_db().await?;

        sqlx::query(
            "UPDATE meetings
             SET album_id = ?, album_name = ?, album_artist = ?, album_art_url = ?,
                 album_spotify_url = ?, picker = ?, meeting_date = ?, meeting_time = ?,
                 meeting_location = ?
             WHERE is_current = 1",
        )
        .bind(req.album_id)
        .bind(req.album_name)
        .bind(req.album_artist)
        .bind(req.album_art_url)
        .bind(req.album_spotify_url)
        .bind(req.picker)
        .bind(req.meeting_date)
        .bind(req.meeting_time)
        .bind(req.meeting_location)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        tracing::info!("POST /api/admin/update-current → ok");
        Ok(())
    }
}

/// Delete a single historical entry by id.
#[post("/api/admin/history/delete")]
pub async fn admin_delete_history_entry(
    admin_token: String,
    id: String,
) -> Result<(), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        ensure_admin_token(&admin_token)?;
        tracing::info!("POST /api/admin/history/delete id=\"{id}\"");
        let pool = get_db().await?;

        sqlx::query("DELETE FROM meetings WHERE id = ? AND is_current = 0")
            .bind(&id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        tracing::info!("POST /api/admin/history/delete id=\"{id}\" → ok");
        Ok(())
    }
}

/// Reorder the member list.
#[post("/api/admin/reorder-members")]
pub async fn admin_reorder_members(
    admin_token: String,
    ordered_names: Vec<String>,
) -> Result<(), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        ensure_admin_token(&admin_token)?;
        tracing::info!(
            "POST /api/admin/reorder-members {} members",
            ordered_names.len()
        );
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

        tracing::info!("POST /api/admin/reorder-members → ok");
        Ok(())
    }
}

/// Verify a member's credentials (name + pre-shared password). Returns Ok if valid.
#[post("/api/member/verify")]
pub async fn verify_member(member_name: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        let _ = (member_name, password);
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    verify_member_password_internal(&member_name, &password).await
}

/// Get the cached track listing for an album. Fetches from Spotify on first call.
#[server]
pub async fn get_album_tracks(album_id: String) -> Result<Vec<AlbumTrack>, ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        let _ = album_id;
        return Ok(Vec::new());
    }

    #[cfg(feature = "server")]
    {
        tracing::debug!("get_album_tracks album_id=\"{album_id}\"");
        let pool = get_db().await?;

        // Return from cache if available.
        let cached: Vec<(String, i64, String, Option<i64>, Option<String>)> = sqlx::query_as(
            "SELECT track_id, track_number, track_name, duration_ms, spotify_url
             FROM album_tracks WHERE album_id = ? ORDER BY track_number",
        )
        .bind(&album_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Return from cache if available and complete (all rows have spotify_url).
        if !cached.is_empty() && cached.iter().all(|(_, _, _, _, url)| url.is_some()) {
            return Ok(cached
                .into_iter()
                .map(
                    |(track_id, track_number, track_name, duration_ms, spotify_url)| AlbumTrack {
                        track_id,
                        track_number: track_number as u32,
                        track_name,
                        duration_ms,
                        spotify_url,
                    },
                )
                .collect());
        }

        // Fetch from Spotify and cache.
        let spotify_client = SPOTIFY_CLIENT.get_or_init(|| tokio::sync::Mutex::new(None));
        let mut spotify_guard = spotify_client.lock().await;

        if spotify_guard.is_none() {
            *spotify_guard = Some(
                spotify::SpotifyClient::from_env()
                    .map_err(|e| ServerFnError::new(e.to_string()))?,
            );
        }

        let client = spotify_guard
            .as_mut()
            .ok_or_else(|| ServerFnError::new("Failed to initialize Spotify client"))?;

        let tracks = client
            .get_album_tracks(&album_id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        // Cache them.
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        for t in &tracks {
            sqlx::query(
                "INSERT OR REPLACE INTO album_tracks (album_id, track_number, track_id, track_name, duration_ms, spotify_url)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&album_id)
            .bind(t.track_number as i64)
            .bind(&t.id)
            .bind(&t.name)
            .bind(t.duration_ms.map(|d| d as i64))
            .bind(&t.spotify_url)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(tracks
            .into_iter()
            .map(|t| AlbumTrack {
                track_id: t.id,
                track_number: t.track_number,
                track_name: t.name,
                duration_ms: t.duration_ms.map(|d| d as i64),
                spotify_url: t.spotify_url,
            })
            .collect())
    }
}

/// Get all album and track reviews for a meeting.
#[server]
pub async fn get_reviews(meeting_id: String) -> Result<Reviews, ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        let _ = meeting_id;
        return Ok(Reviews {
            album_reviews: Vec::new(),
            track_reviews: Vec::new(),
        });
    }

    #[cfg(feature = "server")]
    {
        tracing::debug!("get_reviews meeting_id=\"{meeting_id}\"");
        let pool = get_db().await?;
        use sqlx::Row;

        let album_rows = sqlx::query("SELECT member_name, score FROM album_reviews WHERE meeting_id = ?")
            .bind(&meeting_id)
            .fetch_all(pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let album_reviews = album_rows
            .into_iter()
            .map(|r| AlbumReview {
                member_name: r.get("member_name"),
                score: r.get::<i64, _>("score") as u8,
            })
            .collect();

        let track_rows = sqlx::query(
            "SELECT member_name, track_id, score FROM track_reviews WHERE meeting_id = ?",
        )
        .bind(&meeting_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        let track_reviews = track_rows
            .into_iter()
            .map(|r| TrackReview {
                member_name: r.get("member_name"),
                track_id: r.get("track_id"),
                score: r.get::<i64, _>("score") as u8,
            })
            .collect();

        Ok(Reviews {
            album_reviews,
            track_reviews,
        })
    }
}

/// Submit or update an album-level review.
#[post("/api/review/album")]
pub async fn submit_album_review(
    member_name: String,
    password: String,
    meeting_id: String,
    score: u8,
) -> Result<Reviews, ServerFnError> {
    if score > 10 {
        return Err(ServerFnError::new("Score must be between 0 and 10"));
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = (member_name, password, meeting_id, score);
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        tracing::info!(
            "submit_album_review member=\"{member_name}\" meeting=\"{meeting_id}\" score={score}"
        );
        verify_member_password_internal(&member_name, &password).await?;

        use uuid::Uuid;
        let pool = get_db().await?;

        sqlx::query(
            "INSERT INTO album_reviews (id, meeting_id, member_name, score)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(meeting_id, member_name)
             DO UPDATE SET score = excluded.score, updated_at = datetime('now')",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&meeting_id)
        .bind(&member_name)
        .bind(score as i64)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        tracing::info!("submit_album_review → ok");

        get_reviews(meeting_id).await
    }
}

/// Submit or update a per-track review.
#[post("/api/review/track")]
pub async fn submit_track_review(
    member_name: String,
    password: String,
    meeting_id: String,
    track_id: String,
    score: u8,
) -> Result<Reviews, ServerFnError> {
    tracing::info!("Reviewing track by {member_name} :: {track_id} :: {score}");

    if score > 10 {
        return Err(ServerFnError::new("Score must be between 0 and 10"));
    }

    #[cfg(not(feature = "server"))]
    {
        let _ = (member_name, password, meeting_id, track_id, score);
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        tracing::info!("submit_track_review member=\"{member_name}\" track=\"{track_id}\" meeting=\"{meeting_id}\" score={score}");
        verify_member_password_internal(&member_name, &password).await?;

        use uuid::Uuid;
        let pool = get_db().await?;

        sqlx::query(
            "INSERT INTO track_reviews (id, meeting_id, member_name, track_id, score)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(meeting_id, member_name, track_id)
             DO UPDATE SET score = excluded.score, updated_at = datetime('now')",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&meeting_id)
        .bind(&member_name)
        .bind(&track_id)
        .bind(score as i64)
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        tracing::info!("submit_track_review → ok");
        get_reviews(meeting_id).await
    }
}

#[cfg(feature = "server")]
async fn verify_member_password_internal(
    member_name: &str,
    password: &str,
) -> Result<(), ServerFnError> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use sqlx::Row;

    let pool = get_db().await?;

    let row = sqlx::query("SELECT password_hash FROM members WHERE name = ?")
        .bind(member_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let row = row.ok_or_else(|| ServerFnError::new("Unknown member"))?;
    let hash: Option<String> = row.get("password_hash");

    let hash = hash.ok_or_else(|| {
        ServerFnError::new("No password set for this member — ask an admin to generate one")
    })?;

    let parsed =
        PasswordHash::new(&hash).map_err(|_| ServerFnError::new("Stored hash is invalid"))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ServerFnError::new("Incorrect password"))
}

#[post("/api/admin/spotify/search")]
pub async fn admin_spotify_album_search(
    admin_token: String,
    query: String,
) -> Result<Vec<SpotifyAlbumSearchItem>, ServerFnError> {
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
        ensure_admin_token(&admin_token)?;
        tracing::debug!("POST /api/admin/spotify/search query=\"{search_term}\"");
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

        tracing::debug!("POST /api/admin/spotify/search → {} results", albums.len());
        Ok(albums)
    }
}
/// Generate a new random password for a member, store its Argon2 hash, and return
/// the plain-text password once so the admin can share it with the member.
#[post("/api/admin/member/set-password")]
pub async fn admin_set_member_password(
    admin_token: String,
    member_name: String,
) -> Result<String, ServerFnError> {
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, member_name);
        return Err(ServerFnError::new("Only available on server builds"));
    }

    #[cfg(feature = "server")]
    {
        ensure_admin_token(&admin_token)?;
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };
        use rand::distributions::{Alphanumeric, DistString};

        tracing::info!("POST /api/admin/member/set-password member=\"{member_name}\"");

        let plain: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map_err(|e| ServerFnError::new(format!("Failed to hash password: {e}")))?
            .to_string();

        let pool = get_db().await?;

        let rows_affected = sqlx::query("UPDATE members SET password_hash = ? WHERE name = ?")
            .bind(&hash)
            .bind(&member_name)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ServerFnError::new(format!(
                "Member \"{member_name}\" not found"
            )));
        }

        tracing::info!("POST /api/admin/member/set-password \"{member_name}\" → ok");
        Ok(plain)
    }
}
