use crate::components::stars::AverageStars;
use api::api_models::{AlbumTrack, Reviews};
use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_brands_icons::FaSpotify, Icon};

#[component]
pub fn AggregateScores(
    reviews: ReadSignal<Reviews>,
    tracks: ReadSignal<Vec<AlbumTrack>>,
) -> Element {
    if reviews().album_reviews.is_empty() && reviews().track_reviews.is_empty() && tracks.is_empty()
    {
        return rsx! {};
    }

    let album_avg = if reviews().album_reviews.is_empty() {
        None
    } else {
        let sum: u32 = reviews().album_reviews.iter().map(|r| r.score as u32).sum();
        Some(sum as f32 / reviews().album_reviews.len() as f32)
    };

    // Build per-track averages; always sorted by track number.
    let mut track_data: Vec<(AlbumTrack, Option<f32>, usize)> = tracks()
        .into_iter()
        .map(|track| {
            let scores: Vec<u8> = reviews()
                .track_reviews
                .iter()
                .filter(|r| r.track_id == track.track_id)
                .map(|r| r.score)
                .collect();
            let count = scores.len();
            let avg = if scores.is_empty() {
                None
            } else {
                let sum: u32 = scores.iter().map(|&s| s as u32).sum();
                Some(sum as f32 / count as f32)
            };
            (track, avg, count)
        })
        .collect();
    track_data.sort_by_key(|(t, _, _)| t.track_number);

    rsx! {
        div { class: "card review-aggregate-card",
            h3 { "Gemensamma betyg" }

            if let Some(avg) = album_avg {
                div { class: "review-aggregate-album",
                    span { class: "review-aggregate-label", "Album" }
                    AverageStars { avg }
                    span { class: "review-aggregate-num", {format!("{:.1} / 10", avg)} }
                    span { class: "review-aggregate-count",
                        {format!("({} röster)", reviews().album_reviews.len())}
                    }
                }
            }

            if !track_data.is_empty() {
                div { class: "review-aggregate-tracks",
                    h4 { "Låtar" }
                    for (track , avg_opt , count) in track_data.iter() {
                        div {
                            key: "{track.track_id}",
                            class: "review-aggregate-track-row",
                            span { class: "review-track-num", "{track.track_number}" }
                            span { class: "review-track-name", "{track.track_name}" }
                            span { class: "review-track-spotify-slot",
                                if let Some(ref url) = track.spotify_url {
                                    a {
                                        href: "{url}",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        class: "review-track-spotify-link",
                                        Icon { icon: FaSpotify }
                                    }
                                }
                            }
                            if let Some(avg) = avg_opt {
                                AverageStars { avg: *avg }
                                span { class: "review-aggregate-num", {format!("{:.1}", avg)} }
                                span { class: "review-aggregate-count", "({count})" }
                            } else {
                                AverageStars { avg: 0.0, placeholder: true }
                                span { class: "review-aggregate-num review-aggregate-no-reviews",
                                    "–"
                                }
                                span {}
                            }
                        }
                    }
                }
            }
        }
    }
}
