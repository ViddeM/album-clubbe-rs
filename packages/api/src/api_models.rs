use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub type Name = Arc<str>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub current_album: Album,
    pub next_meeting: Option<Meeting>,
    pub current_person: Name,
    pub members: Vec<Name>,
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
