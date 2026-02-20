use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::icons::fa_regular_icons::FaClock;
use dioxus_free_icons::icons::fi_icons::{FiCalendar, FiExternalLink, FiMapPin, FiMusic, FiUsers};
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
            name: "Vapen & ammunition".into(),
            album_art: "https://i.scdn.co/image/ab67616d0000b27338195e65555be2b7f9324e1c".into(),
            spotify_url: "https://open.spotify.com/album/2DGzTm2R2v3G0IjnxXtP3Y".into(),
            artist: "Kent".into(),
        },
        next_meeting: Some(Meeting {
            date: "Söndag 22/2".into(),
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
                        div { class: "current-album-heading",
                            Icon { icon: FiMusic, class: "note-icon" }
                            h2 { class: "current-album-heading-text", "Nuvarande album" }
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
                UpcomingRotation {
                    current_person: data().current_person,
                    members: data().members,
                }
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

#[component]
fn UpcomingRotation(current_person: ReadSignal<Name>, members: ReadSignal<Vec<Name>>) -> Element {
    let num_members = use_memo(move || members().len() as i64);

    let ordered_members = use_memo(move || {
        let (curr_index, _) = members()
            .iter()
            .enumerate()
            .find(|&(_, name)| name == &current_person())
            .expect("Current person to be in members list");
        let curr_index = curr_index as i64;

        let mut members = members()
            .into_iter()
            .enumerate()
            .map(|(i, n)| (i as i64, n))
            .map(|(i, n)| (((i - curr_index + num_members()) % num_members()), n))
            .collect::<Vec<_>>();

        members.sort_by(|(a, _), (b, _)| a.cmp(b));

        members
            .into_iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>()
    });

    rsx! {
        div { class: "card full-width",
            div { class: "upcoming-header",
                Icon { class: "upcoming-header-icon", icon: FiUsers }
                h2 { class: "upcoming-header-text", "Nästa på tur" }
            }

            div { class: "upcoming-grid",
                for (i , member) in ordered_members().iter().enumerate() {
                    div {
                        key: "{member}",
                        class: "upcoming-grid-element",
                        class: if i == 0 { "upcoming-grid-element-current" } else { "upcoming-grid-element-normal" },
                        div { class: "order-text", "{i + 1}" }
                        div { class: if i == 0 { "current-name-text" } else { "inactive-name-text" },
                            "{member}"
                        }

                        if i == 0 {
                            div { class: "sub-name-text", "Nuvarande" }
                        }
                        if i == 1 {
                            div { class: "sub-name-text", "Nästa" }
                        }
                    }
                }
            }
        }
    }
}
