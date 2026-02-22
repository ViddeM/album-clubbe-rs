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
            name: "Vapen & ammunition".into(),
            album_art: "https://i.scdn.co/image/ab67616d0000b27338195e65555be2b7f9324e1c".into(),
            spotify_url: "https://open.spotify.com/album/2DGzTm2R2v3G0IjnxXtP3Y".into(),
            artist: "Kent".into(),
        },
        next_meeting: Some(Meeting {
            date: "Måndag 16/2".into(),
            time: Some("TBD".into()),
            location: Some("Discord".into()),
        }),
        current_person: "Nox".into(),
        members: vec![
            "Swexbe", "Nox", "Karro", "Vidde", "Stasia", "Dino", "Yoda", "Carl", "Arvid",
        ]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<_>>(),
    };

    Ok(data)
}
