//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;
#[cfg(feature = "server")]
use std::sync::OnceLock;

use crate::api_models::{AdminOverview, AdminPing, Data, SpotifyAlbumSearchItem};

pub mod api_models;

#[cfg(feature = "server")]
static SPOTIFY_CLIENT: OnceLock<tokio::sync::Mutex<Option<spotify::SpotifyClient>>> =
    OnceLock::new();

#[allow(dead_code)]
const ADMIN_TOKEN_ENV: &str = "ADMIN_TOKEN";

#[allow(dead_code)]
fn get_sample_data() -> Data {
    use crate::api_models::{Album, Meeting};

    Data {
        current_album: Album {
            name: "My Chemical Romance".into(),
            album_art: "https://i.scdn.co/image/ab67616d0000b27317f77fab7e8f18d5f9fee4a1".into(),
            spotify_url: "https://open.spotify.com/album/0FZK97MXMm5mUQ8mtudjuK".into(),
            artist: "".into(),
        },
        next_meeting: Some(Meeting {
            date: "Söndag 8/3".into(),
            time: Some("18:00".into()),
            location: Some("Discord".into()),
        }),
        current_person: "Karro".into(),
        members: vec![
            "Swexbe", "Nox", "Karro", "Vidde", "Stasia", "Dino", "Yoda", "Carl", "Arvid",
        ]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<_>>(),
    }
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

/// Get the current album.
#[get("/api/info")]
pub async fn get_current() -> Result<Data, ServerFnError> {
    Ok(get_sample_data())
}

#[post("/api/admin/overview")]
pub async fn get_admin_overview(admin_token: String) -> Result<AdminOverview, ServerFnError> {
    ensure_admin_token(&admin_token)?;

    let data = get_sample_data();
    Ok(AdminOverview {
        members_count: data.members.len(),
        has_scheduled_meeting: data.next_meeting.is_some(),
        current_picker: data.current_person,
    })
}

#[post("/api/admin/ping")]
pub async fn admin_ping(admin_token: String) -> Result<AdminPing, ServerFnError> {
    ensure_admin_token(&admin_token)?;

    Ok(AdminPing {
        status: "ok".to_string(),
    })
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
