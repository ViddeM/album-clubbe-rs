//! Spotify-related server function implementations.

use dioxus::prelude::ServerFnError;

use crate::api_models::{AlbumTrack, SpotifyAlbumSearchItem};

use super::{ensure_admin_token, get_db, get_spotify_client, IntoServerError};

pub(crate) async fn get_album_tracks_impl(
    album_id: String,
) -> Result<Vec<AlbumTrack>, ServerFnError> {
    tracing::debug!("get_album_tracks album_id=\"{album_id}\"");
    let pool = get_db().await?;

    let cached: Vec<(String, i64, String, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT track_id, track_number, track_name, duration_ms, spotify_url
         FROM album_tracks WHERE album_id = ? ORDER BY track_number",
    )
    .bind(&album_id)
    .fetch_all(pool)
    .await
    .server_err()?;

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

    let mut guard = get_spotify_client().await?;
    let client = guard
        .as_mut()
        .ok_or_else(|| ServerFnError::new("Failed to initialize Spotify client"))?;

    let tracks = client
        .get_album_tracks(&album_id)
        .await
        .server_err()?;

    let mut tx = pool.begin().await.server_err()?;

    for t in &tracks {
        sqlx::query(
            "INSERT OR REPLACE INTO album_tracks
                (album_id, track_number, track_id, track_name, duration_ms, spotify_url)
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
        .server_err()?;
    }

    tx.commit().await.server_err()?;

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

pub(crate) async fn admin_spotify_album_search_impl(
    admin_token: String,
    search_term: &str,
) -> Result<Vec<SpotifyAlbumSearchItem>, ServerFnError> {
    ensure_admin_token(&admin_token)?;
    tracing::debug!("POST /api/admin/spotify/search query=\"{search_term}\"");

    let mut guard = get_spotify_client().await?;
    let client = guard
        .as_mut()
        .ok_or_else(|| ServerFnError::new("Failed to initialize Spotify client"))?;

    let albums = client
        .search_albums(search_term)
        .await
        .server_err()?
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
