use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub type Name = Arc<str>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub current_meeting_id: Option<String>,
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
    pub id: String,
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
    pub meeting_date: String,
    pub meeting_time: Option<String>,
    pub meeting_location: Option<String>,
}

/// A single track from an album, cached from Spotify.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumTrack {
    pub track_id: String,
    pub track_number: u32,
    pub track_name: String,
    pub duration_ms: Option<i64>,
    pub spotify_url: Option<String>,
}

/// One member's album-level review score (0–10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumReview {
    pub member_name: String,
    pub score: u8,
}

/// One member's score (0–10) for a single track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackReview {
    pub member_name: String,
    pub track_id: String,
    pub score: u8,
}

/// All reviews for a given meeting (album + individual tracks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reviews {
    pub album_reviews: Vec<AlbumReview>,
    pub track_reviews: Vec<TrackReview>,
}
