use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub type Name = Arc<str>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub current_album: Option<Album>,
    pub next_meeting: Option<Meeting>,
    pub current_person: Option<Name>,
    pub members: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpotifyAlbumSearchItem {
    pub id: String,
    pub name: String,
    pub artists: String,
    pub image_url: Option<String>,
    pub spotify_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Album {
    pub name: String,
    pub artist: String,
    pub album_art: String,
    pub spotify_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meeting {
    pub date: String,
    pub time: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub album_name: String,
    pub album_artist: String,
    pub album_art: String,
    pub spotify_url: String,
    pub picker: String,
    pub recorded_at: String,
    pub meeting_date: Option<String>,
    pub meeting_time: Option<String>,
    pub meeting_location: Option<String>,
}
