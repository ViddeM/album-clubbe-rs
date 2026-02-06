use std::sync::Arc;

use dioxus::prelude::*;

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
                            MusicalNote {}
                            h2 { class: "current-album-heading", "Current Album" }
                        }

                        CurrentAlbumView {
                            album: data().current_album,
                            picked_by: data().current_person,
                        }
                    }
                }

                div {
                    div { class: "card", NextMeeting {
                    } }
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
                    "Picked by "
                    span { "{picked_by()}" }
                }

                // Spotify link
                // TODO: Check if there is a better dioxus way to do this.
                a {
                    href: "{album().spotify_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "spotify-link gap-2",
                    SpotifyLogo {}
                    "Lyssna på Spotify"
                    ExternalLinkIcon {}
                }
            }
        }
    }
}

#[component]
fn MusicalNote() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            fill: "none",
            view_box: "0 0 24 24",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "note-icon",
            path { d: "M9 18V5l12-2v13" }
            circle { cx: "6", cy: "18", r: "3" }
            circle { cx: "18", cy: "16", r: "3" }
        }
    }
}

#[component]
fn SpotifyLogo() -> Element {
    rsx! {
        svg {
            class: "spotify-logo",
            view_box: "0 0 24 24",
            fill: "currentColor",
            path { d: "M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z" }
        }
    }
}

#[component]
fn ExternalLinkIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "external-link-icon",
            path { d: "M15 3h6v6" }
            path { d: "M10 14 21 3" }
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
        }
    }
}

#[component]
fn NextMeeting(next_meeting: Option<Meeting>) -> Element {
    rsx! {
        div { class: "bg-white/10 backdrop-blur-sm rounded-2xl p-8 shadow-2xl border border-white/20 h-full",
            div { class: "flex items-center mb-6 gap-2",
                CalendarIcon { class: "w-5 h-5 text-purple-300" }
                h2 { class: "text-x1 text-purple-200", "Nästa Möte" }
            }
        }
    }
}
