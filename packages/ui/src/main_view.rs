use api::api_models::{Album, Data, Meeting, Name};

use crate::components::stars::{AverageStars, ReviewScore};
use crate::SiteFooter;
use api::{get_current, get_reviews};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::icons::fa_regular_icons::FaClock;
use dioxus_free_icons::icons::fi_icons::{FiCalendar, FiExternalLink, FiMapPin, FiMusic, FiUsers};
use dioxus_free_icons::Icon;

const MAIN_SCSS: Asset = asset!("/assets/styling/main.scss");

#[component]
pub fn Main() -> Element {
    let mut data = use_signal(|| None);
    let mut score = use_signal(|| ReviewScore::Loading);

    use_future(move || async move {
        let current_data = get_current().await;
        if let Err(e) = &current_data {
            eprintln!("Error fetching data: {e}");
        }
        if let Ok(ref d) = current_data {
            if let Some(ref meeting_id) = d.current_meeting_id {
                let meeting_id = meeting_id.clone();
                match get_reviews(meeting_id).await {
                    Ok(r) => {
                        let scores: Vec<u8> = r.album_reviews.iter().map(|rv| rv.score).collect();
                        score.set(ReviewScore::from_scores(&scores));
                    }
                    Err(e) => {
                        eprintln!("Error fetching reviews: {e}");
                        score.set(ReviewScore::NoReviews);
                    }
                }
            } else {
                score.set(ReviewScore::NoReviews);
            }
        }
        data.set(Some(current_data));
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Albumklubben" }
            }

            if let Some(data) = data() {
                if let Ok(data) = data {
                    RenderData { data, score: score() }
                } else {
                    div { "Kunde inte ladda data" }
                }
            } else {
                div { "Laddar..." }
            }

            SiteFooter {}
        }
    }
}

#[component]
fn RenderData(data: ReadSignal<Data>, score: ReviewScore) -> Element {
    rsx! {
        div { class: "first-row row",
            div { class: "double-column",
                div { class: "card",
                    div { class: "current-album-heading",
                        Icon { icon: FiMusic, class: "note-icon" }
                        h2 { class: "current-album-heading-text", "Nuvarande album" }
                    }

                    if let Some(album) = data().current_album {
                        CurrentAlbumView { album, picked_by: data().current_person, score }
                    } else {
                        div { class: "no-album-message",
                            p { "Inget album är valt just nu." }
                            p { "Gå till admin-sidan för att välja ett." }
                        }
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

#[component]
fn CurrentAlbumView(album: Album, picked_by: Option<Name>, score: ReviewScore) -> Element {
    rsx! {
        div { class: "current-album-container gap-6",
            //  Album art
            div { class: "album-art-container",
                img {
                    src: "{album.album_art}",
                    alt: "{album.name} album cover",
                    class: "album-art",
                }
            }

            //Album Info
            div { class: "album-info-container",
                h3 { class: "album-name", "{album.name}" }
                p { class: "album-artist", "{album.artist}" }
                if let Some(picker) = picked_by {
                    p { class: "album-picked-by",
                        "Vald av "
                        span { "{picker}" }
                    }
                }

                // Spotify link
                // TODO: Check if there is a better dioxus way to do this.
                a {
                    href: "{album.spotify_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "spotify-link gap-2",
                    Icon { icon: FaSpotify }
                    "Lyssna på Spotify"
                    Icon { icon: FiExternalLink }
                }

                match score {
                    ReviewScore::Rated { avg, count } => rsx! {
                        div { class: "main-album-score",
                            AverageStars { avg }
                            span { class: "main-album-score-num", {format!("{:.1} / 10", avg)} }
                            span { class: "main-album-score-count", {format!("({} röster)", count)} }
                        }
                    },
                    ReviewScore::NoReviews => rsx! {
                        div { class: "main-album-score",
                            AverageStars { avg: 0.0, placeholder: true }
                            span { class: "main-album-score-count", "Inga betyg ännu" }
                        }
                    },
                    ReviewScore::Loading => rsx! {},
                }

                a { href: "/review", class: "review-link gap-2", "⭐ Recensera albumet" }
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
fn UpcomingRotation(current_person: Option<Name>, members: Vec<Name>) -> Element {
    let num_members = members.len() as i64;

    let ordered_members = {
        let curr_index = current_person
            .as_ref()
            .and_then(|cp| members.iter().position(|name| name == cp))
            .unwrap_or(0) as i64;

        let mut ordered = members
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    ((i as i64 - curr_index + num_members) % num_members),
                    n.clone(),
                )
            })
            .collect::<Vec<_>>();

        ordered.sort_by_key(|(order, _)| *order);
        ordered
            .into_iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>()
    };

    rsx! {
        div { class: "card full-width",
            div { class: "upcoming-header",
                Icon { class: "upcoming-header-icon", icon: FiUsers }
                h2 { class: "upcoming-header-text", "Nästa på tur" }
            }

            div { class: "upcoming-grid",
                for (i , member) in ordered_members.iter().enumerate() {
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
