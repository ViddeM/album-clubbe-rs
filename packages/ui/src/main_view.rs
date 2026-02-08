use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::icons::fa_regular_icons::FaClock;
use dioxus_free_icons::icons::fi_icons::{FiCalendar, FiExternalLink, FiMapPin, FiMusic};
use dioxus_free_icons::Icon;

const MAIN_SCSS: Asset = asset!("/assets/styling/main.scss");

type Name = Arc<str>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Data {
    current_album: Album,
    next_meeting: Option<Meeting>,
    current_person: Name,
    members: Vec<Name>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Album {
    name: String,
    artist: String,
    album_art: String,
    spotify_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Meeting {
    date: String,
    time: Option<String>,
    location: Option<String>,
}

#[component]
pub fn Main() -> Element {
    let data = use_memo(move || Data {
        current_album: Album {
            name: "Sgt. Pepper's Lonely Hearts Club Band (Remastered)".into(),
            album_art: "https://i.scdn.co/image/ab67616d0000b273c92b57b8307e5999ec2fed69".into(),
            spotify_url: "https://open.spotify.com/album/6QaVfG1pHYl1z15ZxkvVDW".into(),
            artist: "The Beatles".into(),
        },
        next_meeting: Some(Meeting {
            date: "Onsdag 15/2".into(),
            time: Some("17:55".into()),
            location: Some("Discord".into()),
        }),
        current_person: "Vidde".into(),
        members: vec![
            "Håll",
            "Karro",
            "Swexboi",
            "Nox",
            "Yoda",
            "Anaztasia",
            "EG",
            "Dino",
        ]
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<_>>(),
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Albumklubben" }
            }

            div { class: "first-row row",
                div { class: "double-column",
                    div { class: "card",
                        div { class: "gap-2 row",
                            Icon { icon: FiMusic, class: "note-icon" }
                            h2 { class: "current-album-heading", "Current Album" }
                        }

                        CurrentAlbumView {
                            album: data().current_album,
                            picked_by: data().current_person,
                        }
                    }
                }

                div {
                    div { class: "card full-height",
                        NextMeeting { next_meeting: data().next_meeting }
                    }
                }
            }

            div { class: "row",
                div { class: "card", "PEPE" }
            }
        }
    }
}

#[component]
fn CurrentAlbumView(album: ReadSignal<Album>, picked_by: ReadSignal<Name>) -> Element {
    rsx! {
        div { class: "current-album-container gap-6",
            //  Album art
            div { class: "album-art-container",
                img {
                    src: "{album().album_art}",
                    alt: "{album().name} album cover",
                    class: "album-art",
                }
            }

            //Album Info
            div { class: "album-info-container",
                h3 { class: "album-name", "{album().name}" }
                p { class: "album-artist", "{album().artist}" }
                p { class: "album-picked-by",
                    "Vald av "
                    span { "{picked_by()}" }
                }

                // Spotify link
                // TODO: Check if there is a better dioxus way to do this.
                a {
                    href: "{album().spotify_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "spotify-link gap-2",
                    Icon { icon: FaSpotify }
                    "Lyssna på Spotify"
                    Icon { icon: FiExternalLink }
                }
            }
        }
    }
}

#[component]
fn NextMeeting(next_meeting: Option<Meeting>) -> Element {
    rsx! {
        div { class: "next-meeting-container",
            div { class: "next-meeting-header",
                Icon { icon: FiCalendar, class: "calendar-icon-heading" }
                h2 { class: "text-x1 text-purple-200", "Nästa Möte" }
            }

            if let Some(meeting) = next_meeting {
                div { class: "next-meeting-info-container",
                    div { class: "next-meeting-row",
                        Icon { icon: FiCalendar, class: "color-purple-400" }
                        div {
                            div { class: "next-meeting-subheading", "Datum" }
                            div { class: "next-meeting-text", "{meeting.date}" }
                        }
                    }

                    div { class: "next-meeting-row",
                        Icon { icon: FaClock, class: "color-purple-400" }
                        div {
                            div { class: "next-meeting-subheading", "Tid" }

                            if let Some(time) = meeting.time {
                                div { class: "next-meeting-text", "{time}" }
                            } else {
                                div { class: "next-meeting-text", "Ej bestämt" }
                            }
                        }
                    }

                    div { class: "next-meeting-row",
                        Icon { icon: FiMapPin, class: "color-purple-400" }
                        div {
                            div { class: "next-meeting-subheading", "Plats" }

                            if let Some(location) = meeting.location {
                                div { class: "next-meeting-text", "{location}" }
                            } else {
                                div { class: "next-meeting-text", "Ej bestämt" }
                            }
                        }
                    }
                }
            } else {
                div { class: "next-meeting-text", "Inget möte inplanerat" }
            }
        }
    }
}
