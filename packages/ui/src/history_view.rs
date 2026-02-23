use api::api_models::HistoryEntry;
use api::get_history;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::icons::fa_regular_icons::FaClock;
use dioxus_free_icons::icons::fi_icons::{FiCalendar, FiExternalLink, FiMapPin};
use dioxus_free_icons::Icon;

const HISTORY_SCSS: Asset = asset!("/assets/styling/history.scss");

#[component]
pub fn History() -> Element {
    let mut entries = use_signal(|| None::<Result<Vec<HistoryEntry>, String>>);

    use_future(move || async move {
        let result = get_history().await.map_err(|e| e.to_string());
        entries.set(Some(result));
    });

    rsx! {
        document::Link { rel: "stylesheet", href: HISTORY_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Historik" }
            }

            match entries() {
                None => rsx! {
                    div { class: "history-loading", "Laddar…" }
                },
                Some(Err(e)) => rsx! {
                    div { class: "history-error", "Kunde inte ladda historik: {e}" }
                },
                Some(Ok(list)) if list.is_empty() => rsx! {
                    div { class: "history-empty",
                        p { "Inga tidigare album än." }
                    }
                },
                Some(Ok(list)) => rsx! {
                    div { class: "history-grid",
                        for entry in list {
                            HistoryCard { entry }
                        }
                    }
                },
            }

            footer { class: "site-footer",
                a { href: "/", "Startsida" }
                a { href: "/admin", "Admin" }
            }
        }
    }
}

#[component]
fn HistoryCard(entry: HistoryEntry) -> Element {
    rsx! {
        div { class: "history-card card",
            div { class: "history-card-art-wrapper",
                img {
                    class: "history-card-art",
                    src: "{entry.album_art}",
                    alt: "{entry.album_name} album cover",
                }
            }

            div { class: "history-card-info",
                div { class: "history-card-header",
                    div { class: "history-card-titles",
                        p { class: "history-card-album", "{entry.album_name}" }
                        p { class: "history-card-artist", "{entry.album_artist}" }
                    }
                }

                p { class: "history-card-picker",
                    "Vald av "
                    span { "{entry.picker}" }
                }

                div { class: "history-card-meeting",
                    div { class: "history-card-meeting-row",
                        Icon {
                            icon: FiCalendar,
                            class: "history-card-meeting-icon",
                        }
                        span { class: if entry.meeting_date.is_none() { "history-card-meeting-unset" } else { "" },
                            {entry.meeting_date.as_deref().unwrap_or("Ej angivet")}
                        }
                    }
                    div { class: "history-card-meeting-row",
                        Icon { icon: FaClock, class: "history-card-meeting-icon" }
                        span { class: if entry.meeting_time.is_none() { "history-card-meeting-unset" } else { "" },
                            {entry.meeting_time.as_deref().unwrap_or("Ej angivet")}
                        }
                    }
                    div { class: "history-card-meeting-row",
                        Icon {
                            icon: FiMapPin,
                            class: "history-card-meeting-icon",
                        }
                        span { class: if entry.meeting_location.is_none() { "history-card-meeting-unset" } else { "" },
                            {entry.meeting_location.as_deref().unwrap_or("Ej angivet")}
                        }
                    }
                }

                a {
                    href: "{entry.spotify_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "history-spotify-link",
                    Icon { icon: FaSpotify }
                    "Spotify"
                    Icon { icon: FiExternalLink }
                }
            }
        }
    }
}
