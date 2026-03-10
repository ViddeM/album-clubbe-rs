use api::api_models::{AlbumTrack, Reviews};
use dioxus::{core::EventHandler, prelude::*};
use dioxus_free_icons::{icons::fa_brands_icons::FaSpotify, Icon};
use std::collections::HashMap;

#[component]
pub fn ReviewLoggedInView(
    logged_in_as: ReadSignal<String>,
    reviews: ReadSignal<Reviews>,
    tracks: ReadSignal<Vec<AlbumTrack>>,
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

    // Pre-fill existing per-track ratings for this member
    let track_ratings = use_memo(move || {
        let mut map: HashMap<String, u8> = HashMap::new();

        for tr in reviews()
            .track_reviews
            .iter()
            .filter(|r| r.member_name == logged_in_as())
        {
            map.insert(tr.track_id.clone(), tr.score);
        }

        map
    });

    // Local staged changes (not yet submitted to API)
    let mut staged_album: Signal<u8> = use_signal(|| album_rating());
    let mut staged_track_scores: Signal<HashMap<String, u8>> = use_signal(|| track_ratings());

    // Keep staged values in sync when server reviews change (e.g. after submit)
    // Clone once and move into effect to avoid repeated work and shadowing.
    let album_rating_clone = album_rating.clone();
    let track_ratings_clone = track_ratings.clone();
    let mut staged_album_clone = staged_album.clone();
    let mut staged_track_scores_clone = staged_track_scores.clone();

    use_effect(move || {
        // Initialize staged values from server-provided ratings whenever
        // the server-side memos change.
        staged_album_clone.set(album_rating_clone());
        staged_track_scores_clone.set(track_ratings_clone());
    });

    // Has local changes compared to server
    let has_changes = use_memo(move || {
        if staged_album() != album_rating() {
            return true;
        }

        if staged_track_scores() != track_ratings() {
            return true;
        }

        false
    });

    // Submit staged changes: call existing callbacks for changed entries
    let submit_staged = {
        let staged_album = staged_album.clone();
        let staged_track_scores = staged_track_scores.clone();
        let album_rating = album_rating.clone();
        let track_ratings = track_ratings.clone();
        let review_album = review_album.clone();
        let review_track = review_track.clone();
        let reset_errors = reset_errors.clone();

        use_callback(move |()| {
            reset_errors(());

            // Submit album if changed
            if staged_album() != album_rating() {
                review_album(staged_album());
            }

            // Submit each changed track
            for (tid, &score) in staged_track_scores().iter() {
                let server_score = track_ratings().get(tid).copied().unwrap_or(0);
                if score != server_score {
                    review_track((tid.clone(), score));
                }
            }
        })
    };

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
                    // Visual indicator for change direction
                    {
                        let album_change = staged_album() as i8 - album_rating() as i8;
                        let change_class = if album_change > 0 {
                            "changed-up"
                        } else if album_change < 0 {
                            "changed-down"
                        } else {
                            ""
                        };

                        rsx! {
                            div { class: "star-wrap {change_class}",
                                StarRating {
                                    score: staged_album(),
                                    on_change: move |s| {
                                        staged_album.set(s);
                                    },
                                }
                            }
                        }
                    }
                    span { class: "review-score-text", "{staged_album()} / 10" }
                }

                if let Some(e) = album_review_error() {
                    p { class: "review-error", "Fel: {e}" }
                }
            }

            // ── Track reviews ────────────────────────────
            div { class: "card review-section",
                h3 { "Låtbetyg" }
                p { class: "review-section-hint", "Sätt ett betyg för varje låt." }

                div { class: "review-track-list",
                    for track in tracks().iter() {
                        {
                            let server_score = *track_ratings().get(&track.track_id).unwrap_or(&0);
                            let staged_score = *staged_track_scores().get(&track.track_id).unwrap_or(&0);
                            let change = staged_score as i8 - server_score as i8;
                            let tc = if change > 0 {
                                "changed-up"
                            } else if change < 0 {
                                "changed-down"
                            } else {
                                ""
                            };
                            rsx! {
                                div { class: "track-wrap {tc}",
                                    TrackRatingRow {
                                        key: "{track.track_id}",
                                        track: track.clone(),
                                        score: staged_score,
                                        on_change: {
                                            let tid = track.track_id.clone();
                                            let mut staged_track_scores = staged_track_scores.clone();
                                            move |s| {
                                                reset_errors(());
                                                let mut new = staged_track_scores().clone();
                                                new.insert(tid.clone(), s);
                                                staged_track_scores.set(new);
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }

                // Submit / reset controls
                div { class: "review-submit-row",
                    button {
                        class: "review-submit-btn",
                        disabled: (!has_changes()).then(|| "disabled"),
                        onclick: move |_| {
                            submit_staged(());
                        },
                        "Spara recensioner"
                    }
                    button {
                        class: "review-reset-btn",
                        onclick: move |_| {
                            reset_errors(());
                            staged_album.set(album_rating());
                            staged_track_scores.set(track_ratings());
                        },
                        "Återställ"
                    }
                }

                if let Some(e) = track_review_error() {
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
