use crate::components::stars::{AverageStars, ReviewScore};
use crate::SiteFooter;
use api::api_models::HistoryEntry;
use api::{get_history, get_reviews};
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
                Some(Ok(list)) => {
                    let groups = group_history_by_month(list);
                    rsx! {
                        div { class: "history-timeline",
                            for (label , entries) in groups {
                                div { class: "history-group",
                                    div { class: "history-group-header",
                                        h2 { class: "history-group-heading", "{label}" }
                                        div { class: "history-group-line" }
                                    }
                                    div { class: "history-grid",
                                        for entry in entries {
                                            HistoryCard { entry }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SiteFooter {}
        }
    }
}

fn month_label_from_date(date_str: &str) -> String {
    let mut parts = date_str.splitn(3, '-');
    let year = parts.next().unwrap_or("?");
    let month = parts.next().unwrap_or("?");
    let month_name = match month {
        "01" => "Januari",
        "02" => "Februari",
        "03" => "Mars",
        "04" => "April",
        "05" => "Maj",
        "06" => "Juni",
        "07" => "Juli",
        "08" => "Augusti",
        "09" => "September",
        "10" => "Oktober",
        "11" => "November",
        "12" => "December",
        _ => "Okänt",
    };
    format!("{} {}", month_name, year)
}

fn group_history_by_month(mut entries: Vec<HistoryEntry>) -> Vec<(String, Vec<HistoryEntry>)> {
    entries.sort_unstable_by(|a, b| b.meeting_date.cmp(&a.meeting_date));
    let mut groups: Vec<(String, Vec<HistoryEntry>)> = Vec::new();
    for entry in entries {
        let label = month_label_from_date(&entry.meeting_date);
        if groups.last().map(|(l, _)| l == &label).unwrap_or(false) {
            groups.last_mut().unwrap().1.push(entry);
        } else {
            groups.push((label, vec![entry]));
        }
    }
    groups
}

#[component]
fn HistoryCard(entry: HistoryEntry) -> Element {
    let mut score = use_signal(|| ReviewScore::Loading);
    let meeting_id = entry.id.clone();

    use_future(move || {
        let meeting_id = meeting_id.clone();
        async move {
            match get_reviews(meeting_id).await {
                Ok(reviews) => {
                    let scores: Vec<u8> = reviews.album_reviews.iter().map(|r| r.score).collect();
                    score.set(ReviewScore::from_scores(&scores));
                }
                Err(_) => score.set(ReviewScore::NoReviews),
            }
        }
    });

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
                        span { "{entry.meeting_date}" }
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

                match score() {
                    ReviewScore::Rated { avg, count } => rsx! {
                        div { class: "history-card-score",
                            AverageStars { avg }
                            span { class: "history-card-score-num", {format!("{:.1}", avg)} }
                            span { class: "history-card-score-count", {format!("({count})")} }
                        }
                    },
                    ReviewScore::NoReviews => rsx! {
                        div { class: "history-card-score",
                            AverageStars { avg: 0.0, placeholder: true }
                            span { class: "history-card-score-count", "Inga betyg" }
                        }
                    },
                    ReviewScore::Loading => rsx! {},
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
