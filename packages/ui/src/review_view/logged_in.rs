use api::api_models::{AlbumTrack, Reviews};
use dioxus::{core::EventHandler, prelude::*};
use dioxus_free_icons::{icons::fa_brands_icons::FaSpotify, Icon};
use std::collections::HashMap;

/// Returns the CSS class name reflecting whether a score moved up, down, or is unchanged.
fn change_class(staged: u8, server: u8) -> &'static str {
    match (staged as i8).cmp(&(server as i8)) {
        std::cmp::Ordering::Greater => "changed-up",
        std::cmp::Ordering::Less => "changed-down",
        std::cmp::Ordering::Equal => "",
    }
}

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
    let album_rating = use_memo(move || {
        reviews()
            .album_reviews
            .iter()
            .find(|r| r.member_name == logged_in_as())
            .map(|r| r.score)
            .unwrap_or(0)
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
    // Note: clippy suggests `use_signal(album_rating)` but Memo<T> is not FnOnce() — closure is required.
    #[allow(clippy::redundant_closure)]
    let mut staged_album: Signal<u8> = use_signal(|| album_rating());
    #[allow(clippy::redundant_closure)]
    let mut staged_track_scores: Signal<HashMap<String, u8>> = use_signal(|| track_ratings());

    // Keep staged values in sync when server reviews change (e.g. after submit)
    use_effect(move || {
        // Initialize staged values from server-provided ratings whenever
        // the server-side memos change.
        staged_album.set(album_rating());
        staged_track_scores.set(track_ratings());
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
    let submit_staged = use_callback(move |()| {
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
    });

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
                    {
                        let change_class = change_class(staged_album(), album_rating());

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
                            let tc = change_class(staged_score, server_score);
                            rsx! {
                                div { class: "track-wrap {tc}",
                                    TrackRatingRow {
                                        key: "{track.track_id}",
                                        track: track.clone(),
                                        score: staged_score,
                                        on_change: {
                                            let tid = track.track_id.clone();
                                            let mut staged_track_scores = staged_track_scores;
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
                        disabled: (!has_changes()).then_some("disabled"),
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
