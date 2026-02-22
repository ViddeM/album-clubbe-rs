//! This crate contains all shared fullstack server functions.
use dioxus::prelude::*;

use crate::api_models::Data;

pub mod api_models;

/// Get the current album.
#[get("/api/info")]
pub async fn get_current() -> Result<Data, ServerFnError> {
    use crate::api_models::{Album, Meeting};

    let data = Data {
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
    };

    Ok(data)
}
