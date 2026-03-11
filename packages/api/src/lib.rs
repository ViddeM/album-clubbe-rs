//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

use crate::api_models::{AlbumTrack, Data, HistoryEntry, Reviews, SetCurrentRequest, SpotifyAlbumSearchItem};

pub mod api_models;

#[cfg(feature = "server")]
mod db;

#[cfg(feature = "server")]
mod server;

#[cfg(feature = "server")]
pub use server::init_db;

/// Get the current album, next meeting and member list.
#[get("/api/info")]
pub async fn get_current() -> Result<Data, ServerFnError> {
    #[cfg(feature = "server")]
    { server::get_current_impl().await }
    #[cfg(not(feature = "server"))]
    { Err(ServerFnError::new("Only available on server builds")) }
}

/// Get all past (non-current) meetings, ordered by meeting date ascending.
#[get("/api/history")]
pub async fn get_history() -> Result<Vec<HistoryEntry>, ServerFnError> {
    #[cfg(feature = "server")]
    { server::get_history_impl().await }
    #[cfg(not(feature = "server"))]
    { Err(ServerFnError::new("Only available on server builds")) }
}

/// Set the current album, meeting info and picker. Archives the previous state to history.
#[post("/api/admin/set-current")]
pub async fn admin_set_current(
    admin_token: String,
    req: SetCurrentRequest,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    { server::admin_set_current_impl(admin_token, req).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, req);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Update the current meeting / album in-place (without archiving to history).
#[post("/api/admin/update-current")]
pub async fn admin_update_current(
    admin_token: String,
    req: SetCurrentRequest,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    { server::admin_update_current_impl(admin_token, req).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, req);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Delete a single historical entry by id.
#[post("/api/admin/history/delete")]
pub async fn admin_delete_history_entry(
    admin_token: String,
    id: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    { server::admin_delete_history_entry_impl(admin_token, id).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, id);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Reorder the member list.
#[post("/api/admin/reorder-members")]
pub async fn admin_reorder_members(
    admin_token: String,
    ordered_names: Vec<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    { server::admin_reorder_members_impl(admin_token, ordered_names).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, ordered_names);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Verify a member's credentials (name + pre-shared password). Returns Ok if valid.
#[post("/api/member/verify")]
pub async fn verify_member(member_name: String, password: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "server")]
    { server::verify_member_password_internal(&member_name, &password).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (member_name, password);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Get the cached track listing for an album. Fetches from Spotify on first call.
#[server]
pub async fn get_album_tracks(album_id: String) -> Result<Vec<AlbumTrack>, ServerFnError> {
    #[cfg(feature = "server")]
    { server::get_album_tracks_impl(album_id).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = album_id;
        Ok(Vec::new())
    }
}

/// Get all album and track reviews for a meeting.
#[server]
pub async fn get_reviews(meeting_id: String) -> Result<Reviews, ServerFnError> {
    #[cfg(feature = "server")]
    { server::get_reviews_impl(meeting_id).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = meeting_id;
        Ok(Reviews {
            album_reviews: Vec::new(),
            track_reviews: Vec::new(),
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
    #[cfg(feature = "server")]
    { server::submit_album_review_impl(member_name, password, meeting_id, score).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (member_name, password, meeting_id, score);
        Err(ServerFnError::new("Only available on server builds"))
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
    if score > 10 {
        return Err(ServerFnError::new("Score must be between 0 and 10"));
    }
    #[cfg(feature = "server")]
    { server::submit_track_review_impl(member_name, password, meeting_id, track_id, score).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (member_name, password, meeting_id, track_id, score);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Search Spotify for albums matching a query.
#[post("/api/admin/spotify/search")]
pub async fn admin_spotify_album_search(
    admin_token: String,
    query: String,
) -> Result<Vec<SpotifyAlbumSearchItem>, ServerFnError> {
    let search_term = query.trim();
    if search_term.is_empty() {
        return Ok(Vec::new());
    }
    #[cfg(feature = "server")]
    { server::admin_spotify_album_search_impl(admin_token, search_term).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, search_term);
        Err(ServerFnError::new("Only available on server builds"))
    }
}

/// Generate a new random password for a member, store its Argon2 hash, and return
/// the plain-text password once so the admin can share it with the member.
#[post("/api/admin/member/set-password")]
pub async fn admin_set_member_password(
    admin_token: String,
    member_name: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "server")]
    { server::admin_set_member_password_impl(admin_token, member_name).await }
    #[cfg(not(feature = "server"))]
    {
        let _ = (admin_token, member_name);
        Err(ServerFnError::new("Only available on server builds"))
    }
}
