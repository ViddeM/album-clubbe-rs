use api::api_models::{AlbumTrack, Reviews};
use dioxus::{core::EventHandler, prelude::*};
use dioxus_free_icons::{icons::fa_brands_icons::FaSpotify, Icon};

#[component]
pub fn ReviewLoggedInView(
    logged_in_as: ReadSignal<String>,
    reviews: ReadSignal<Reviews>,
    review_album: Callback<u8, ()>,
    review_track: Callback<(String, u8), ()>,
    logout: Callback<(), ()>,
    album_review_error: ReadSignal<Option<String>>,
    track_review_error: ReadSignal<Option<String>>,
    reset_errors: Callback<(), ()>,
) -> Element {
    // // Pre-fill existing ratings
    // if let Some(ar) = reviews()
    //     .album_reviews
    //     .iter()
    //     .find(|r| r.member_name == *name)
    // {
    //     album_rating.set(ar.score);
    // }

    // let mut map = HashMap::new();
    // for track in reviews()
    //     .track_reviews
    //     .iter()
    //     .filter(|r| r.member_name == *name)
    // {
    //     map.insert(track.track_id.clone(), track.score);
    // }
    // track_ratings.set(map);
    // logged_in_as.set(Some(name));
    // login_error.set(None);

    let album_rating = use_memo(move || {
        reviews()
            .album_reviews
            .iter()
            .find(|r| r.member_name == logged_in_as())
            .expect("Current album to exist")
            .score
    });

    // let review_track = use_callback(move |(name, password, meeting_id, track_id, review)| {
    //     spawn(async move {
    //         let result = submit_track_review(name, password, meeting_id, track_id, review).await;

    //         match result {
    //             Ok(r) => {
    //                 update_reviews(r);
    //             }
    //             Err(err) => {
    //                 album_review_error.set(Some(err.to_string()));
    //             }
    //         }
    //     });
    // });

    rsx! {
        div { class: "review-form-container",
            // ── Logout ────────────────────────────
            div { class: "review-logged-banner",
                span { "Inloggad som " }
                strong { "{logged_in_as}" }
                button {
                    class: "review-logout-btn",
                    onclick: move |_| {
                        reset_errors(());
                        logout(());
                    },
                    "Logga ut"
                }
            }

            // ── Album review ────────────────────────────
            div { class: "card review-section",
                h3 { "Albumbetyg" }
                p { class: "review-section-hint", "Ge albumet ett betyg från 1 till 10." }
                div { class: "review-star-row",
                    StarRating {
                        score: album_rating(),
                        on_change: move |s| {
                            review_album(s);
                        },
                    }
                    span { class: "review-score-text", "{album_rating()} / 10" }
                }

                if let Some(e) = album_review_error() {
                    p { class: "review-error", "Fel: {e}" }
                }
            }
        }
    }

    // rsx! {
    //     div { class: "review-form-container",
    //         div { class: "review-logged-banner",
    //             span { "Inloggad som " }
    //             strong { "{logged_name}" }
    //             button {
    //                 class: "review-logout-btn",
    //                 onclick: move |_| {
    //                     logged_in_as.set(None);
    //                     album_review_error.set(None);
    //                     track_review_error.set(None);
    //                 },
    //                 "Logga ut"
    //             }
    //         }

    //         // ── Album review ────────────────────────────
    //         div { class: "card review-section",
    //             h3 { "Albumbetyg" }
    //             p { class: "review-section-hint", "Ge albumet ett betyg från 1 till 10." }
    //             div { class: "review-star-row",
    //                 StarRating {
    //                     score: album_rating(),
    //                     on_change: move |s| {
    //                         album_rating.set(s);
    //                         album_review_error.set(None);

    //                         let name = logged_in_as().unwrap_or_default();
    //                         let pw = password();
    //                         let mid = meeting_id();

    //                         spawn(async move {
    //                             let result = submit_album_review(name, pw, mid.clone(), s)
    //                                 .await
    //                                 .map_err(|e| e.to_string());

    //                             if result.is_ok() {
    //                                 load_reviews(());
    //                             }
    //                             album_submit.set(Some(result));

    //                         });
    //                     },
    //                 }
    //                 span { class: "review-score-text", "{album_rating()} / 10" }
    //             }

    //             if let Some(Err(ref e)) = album_submit() {
    //                 p { class: "review-error", "Fel: {e}" }
    //             }
    //         }

    //         // ── Track reviews ────────────────────────────
    //         match tracks() {
    //             None => rsx! {
    //                 div { class: "review-tracks-loading", "Laddar låtar…" }
    //             },
    //             Some(Err(ref e)) => rsx! {
    //                 div { class: "review-error", "Kunde inte ladda låtar: {e}" }
    //             },
    //             Some(Ok(ref track_list)) if track_list.is_empty() => rsx! {},
    //             Some(Ok(ref track_list)) => rsx! {
    //                 div { class: "card review-section",
    //                     h3 { "Låtbetyg" }
    //                     p { class: "review-section-hint", "Sätt ett betyg för varje låt." }

    //                     div { class: "review-track-list",
    //                         for track in track_list.iter() {
    //                             TrackRatingRow {
    //                                 key: "{track.track_id}",
    //                                 track: track.clone(),
    //                                 score: track_ratings().get(&track.track_id).copied().unwrap_or(0),
    //                                 on_change: {
    //                                     let tid = track.track_id.clone();
    //                                     move |s| {
    //                                         track_submit.set(None);

    //                                         let name = logged_in_as().unwrap_or_default();
    //                                         let pw = password();
    //                                         let mid = meeting_id();
    //                                         let t = tid.clone();

    //                                         spawn(async move {
    //                                             let result = submit_track_review(name, pw, mid.clone(), t.clone(), s)
    //                                                 .await
    //                                                 .map_err(|e| e.to_string());

    //                                             load_reviews(());
    //                                             track_submit.set(Some(result));
    //                                         });
    //                                     }
    //                                 },
    //                             }
    //                         }
    //                     }

    //                     if let Some(Err(ref e)) = track_submit() {
    //                         p { class: "review-error", "Fel: {e}" }
    //                     }
    //                 }
    //             },
    //         }
    //     }
    // }
}

#[component]
fn TrackRatingRow(track: AlbumTrack, score: u8, on_change: EventHandler<u8>) -> Element {
    rsx! {
        div { class: "review-track-row",
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
            div { class: "review-track-rating",
                StarRating { score, on_change }
                span { class: "review-track-score-text",
                    if score == 0 {
                        "–"
                    } else {
                        "{score}"
                    }
                }
            }
        }
    }
}

#[component]
fn StarRating(score: u8, on_change: EventHandler<u8>) -> Element {
    let mut hover: Signal<Option<u8>> = use_signal(|| None);

    let display = hover().unwrap_or(score);

    rsx! {
        div {
            class: "star-rating",
            role: "group",
            "aria-label": format!("Betyg {} av 10", score),
            onmouseleave: move |_| hover.set(None),

            for star in 1u8..=5 {
                {
                    let left_score = star * 2 - 1;
                    let right_score = star * 2;

                    // Determine fill level: "full", "half", or "empty"
                    let fill = if display >= right_score {
                        "full"
                    } else if display >= left_score {
                        "half"
                    } else {
                        "empty"
                    };

                    rsx! {
                        div {
                            key: "{star}",
                            class: "star-slot star-{fill}",
                            "aria-label": "Stjärna {star}",

                            // Full star glyph (background, always shown)
                            span { class: "star-bg", "★" }

                            // Left half click-zone
                            div {
                                class: "star-half-zone star-half-left",
                                "aria-label": "{left_score} av 10",
                                onmouseenter: move |_| hover.set(Some(left_score)),
                                onclick: move |_| {
                                    let new = if score == left_score { 0 } else { left_score };
                                    on_change.call(new);
                                    hover.set(None);
                                },
                            }

                            // Right half click-zone
                            div {
                                class: "star-half-zone star-half-right",
                                "aria-label": "{right_score} av 10",
                                onmouseenter: move |_| hover.set(Some(right_score)),
                                onclick: move |_| {
                                    let new = if score == right_score { left_score } else { right_score };
                                    on_change.call(new);
                                    hover.set(None);
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
