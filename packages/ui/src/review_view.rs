use std::collections::HashMap;

use crate::SiteFooter;
use api::api_models::{Album, AlbumTrack, Data, Name, Reviews};
use api::{get_album_tracks, get_current, get_reviews, submit_album_review, verify_member};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::icons::fi_icons::FiExternalLink;
use dioxus_free_icons::Icon;

const REVIEW_SCSS: Asset = asset!("/assets/styling/review.scss");

// ─────────────────────────────────────────────────────────────────────────────
// Page
// ─────────────────────────────────────────────────────────────────────────────

#[component]
pub fn Review() -> Element {
    let mut page_data = use_signal(|| None::<Result<Data, String>>);
    let mut tracks = use_signal(|| None::<Result<Vec<AlbumTrack>, String>>);
    let mut reviews = use_signal(|| None::<Result<Reviews, String>>);

    // Current meeting ID stored as a signal so closures can capture it.
    let mut meeting_id: Signal<String> = use_signal(String::new);

    // Auth state
    let member_name = use_signal(String::new);
    let password = use_signal(String::new);
    let mut logged_in_as = use_signal(|| None::<String>);
    let mut login_error = use_signal(|| None::<String>);

    // Review input state (populated from existing reviews after login)
    let mut album_rating: Signal<u8> = use_signal(|| 0u8);
    let mut track_ratings: Signal<HashMap<String, u8>> = use_signal(HashMap::new);

    // Submit states
    let mut album_submit = use_signal(|| None::<Result<(), String>>);
    let mut track_submit = use_signal(|| None::<Result<(), String>>);

    let perform_login = use_callback(move |_: ()| {
        let name = member_name();
        let pw = password();
        login_error.set(None);
        spawn(async move {
            match verify_member(name.clone(), pw).await {
                Ok(()) => {
                    // Pre-fill existing ratings
                    if let Some(Ok(ref rev)) = reviews() {
                        if let Some(ar) = rev.album_reviews.iter().find(|r| r.member_name == *name)
                        {
                            album_rating.set(ar.score);
                        }
                        let mut map = HashMap::new();
                        for tr in rev.track_reviews.iter().filter(|r| r.member_name == *name) {
                            map.insert(tr.track_id.clone(), tr.score);
                        }
                        track_ratings.set(map);
                    }
                    logged_in_as.set(Some(name));
                    login_error.set(None);
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                }
            }
        });
    });

    // Load current album on mount
    use_future(move || async move {
        let data = get_current().await.map_err(|e| e.to_string());
        let album_id = data
            .as_ref()
            .ok()
            .and_then(|d| d.current_album.as_ref())
            .map(|a| a.id.clone());
        let mid = data
            .as_ref()
            .ok()
            .and_then(|d| d.current_meeting_id.clone())
            .unwrap_or_default();
        meeting_id.set(mid.clone());
        page_data.set(Some(data));

        // Load tracks and reviews if we have an album
        if let Some(aid) = album_id {
            if !mid.is_empty() {
                let t = get_album_tracks(aid).await;
                let r = get_reviews(mid).await;
                tracks.set(Some(t.map_err(|e| e.to_string())));
                reviews.set(Some(r.map_err(|e| e.to_string())));
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: REVIEW_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Recensera" }
            }

            match page_data() {
                None => rsx! {
                    div { class: "review-loading", "Laddar…" }
                },
                Some(Err(e)) => rsx! {
                    div { class: "review-error", "Kunde inte ladda data: {e}" }
                },
                Some(Ok(data)) => {
                    let Some(album) = data.current_album.clone() else {
                        return rsx! {
                            div { class: "review-empty card",
                                p { "Inget album att recensera just nu." }
                                p {
                                    a { href: "/", "Gå till startsidan" }
                                }
                            }
                        };
                    };

                    let members = data.members.clone();

                    rsx! {
                        CurrentAlbumView { album, picked_by: data.current_person }

                        // ── Aggregate scores ────────────────────────────────
                        if let Some(Ok(ref rev)) = reviews() {
                            AggregateScores {
                                reviews: rev.clone(),
                                tracks: tracks().and_then(|t| t.ok()).unwrap_or_default(),
                            }
                        }

                        // ── Login ───────────────────────────────────────────
                        if logged_in_as().is_none() {
                            ReviewLoginView {
                                logged_in_as,
                                member_name,
                                password,
                                login_error,
                                members,
                                reviews,
                                perform_login,
                            }
                        }

                        // ── Review form (shown when logged in) ──────────────
                        if let Some(logged_name) = logged_in_as() {
                            ReviewLoggedInView {
                                logged_name,
                                logged_in_as,
                                album_rating,
                                album_submit,
                                track_submit,
                                password,
                                reviews,
                                tracks,
                                track_ratings,
                                meeting_id,
                            }
                        }
                    }
                }
            }

            SiteFooter {}
        }
    }
}

#[component]
fn CurrentAlbumView(album: ReadSignal<Album>, picked_by: ReadSignal<Option<Name>>) -> Element {
    rsx! {
        // ── Album info ──────────────────────────────────────
        div { class: "card review-album-card",
            div { class: "review-album-art-wrap",
                img {
                    class: "review-album-art",
                    src: "{album().album_art}",
                    alt: "{album().name} album cover",
                }
            }
            div { class: "review-album-info",
                h2 { class: "review-album-name", "{album().name}" }
                p { class: "review-album-artist", "{album().artist}" }
                if let Some(ref picker) = picked_by() {
                    p { class: "review-album-picker",
                        "Vald av "
                        span { class: "review-album-picker-name", "{picker}" }
                    }
                }
                a {
                    href: "{album().spotify_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "review-spotify-link gap-2",
                    Icon { icon: FaSpotify }
                    "Lyssna"
                    Icon { icon: FiExternalLink }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Aggregate scores (read-only)
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn AggregateScores(reviews: ReadSignal<Reviews>, tracks: ReadSignal<Vec<AlbumTrack>>) -> Element {
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
                    span { class: "review-aggregate-num", {format!("{:.1} / 5.0", (avg as f32) / 2.0)} }
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

// ─────────────────────────────────────────────────────────────────────────────
// Track rating row
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Interactive star rating input (0–10 via 5 half-stars)
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Read-only star display for aggregates
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn AverageStars(avg: f32, #[props(default = false)] placeholder: bool) -> Element {
    // Round to nearest integer on the 0–10 scale (each integer = one half-star)
    let display_score = avg.round() as u8;
    let class = if placeholder {
        "star-rating star-rating-readonly star-rating-placeholder"
    } else {
        "star-rating star-rating-readonly"
    };

    rsx! {
        div {
            class: "{class}",
            "aria-label": format!("Genomsnitt {:.1} av 10", avg),
            for star in 1u8..=5 {
                {
                    let left_score = star * 2 - 1;
                    let right_score = star * 2;
                    let fill = if display_score >= right_score {
                        "full"
                    } else if display_score >= left_score {
                        "half"
                    } else {
                        "empty"
                    };
                    rsx! {
                        div { key: "{star}", class: "star-slot star-{fill}",
                            span { class: "star-bg", "★" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReviewLoginView(
    logged_in_as: Signal<Option<String>>,
    member_name: Signal<String>,
    password: Signal<String>,
    login_error: Signal<Option<String>>,
    members: ReadSignal<Vec<Name>>,
    reviews: ReadSignal<Option<Result<Reviews, String>>>,
    perform_login: Callback<()>,
) -> Element {
    rsx! {
        div { class: "card review-login-card",
            h2 { "Logga in för att recensera" }
            p { class: "review-login-hint", "Välj ditt namn och ange ditt lösenord." }

            div { class: "review-login-fields",
                div { class: "review-field",
                    label { class: "review-label", r#for: "review-member", "Namn" }
                    select {
                        id: "review-member",
                        value: "{member_name}",
                        onchange: move |e| {
                            member_name.set(e.value());
                            login_error.set(None);
                        },
                        option {
                            value: "",
                            disabled: true,
                            selected: member_name().is_empty(),
                            "Välj…"
                        }
                        for m in members.iter() {
                            option {
                                value: "{m}",
                                selected: member_name() == m.as_ref(),
                                "{m}"
                            }
                        }
                    }
                }

                div { class: "review-field",
                    label { class: "review-label", r#for: "review-pw", "Lösenord" }
                    input {
                        id: "review-pw",
                        r#type: "password",
                        placeholder: "Ditt lösenord",
                        value: "{password}",
                        oninput: move |e| {
                            password.set(e.value());
                            login_error.set(None);
                        },
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                let name = member_name();
                                let pw = password();
                                if !name.is_empty() && !pw.is_empty() {
                                    spawn(async move {
                                        match verify_member(name.clone(), pw).await {
                                            Ok(()) => {
                                                logged_in_as.set(Some(name));
                                                login_error.set(None);
                                            }
                                            Err(e) => {
                                                login_error.set(Some(e.to_string()));
                                            }
                                        }
                                    });
                                }
                            }
                        },
                    }
                }
            }

            if let Some(err) = login_error() {
                p { class: "review-error", "{err}" }
            }

            button {
                class: "review-button",
                disabled: member_name().is_empty() || password().is_empty(),
                onclick: move |_| perform_login(()),
                "Logga in"
            }
        }
    }
}

#[component]
fn ReviewLoggedInView(
    logged_name: ReadSignal<String>,
    logged_in_as: Signal<Option<String>>,
    album_rating: Signal<u8>,
    album_submit: Signal<Option<Result<(), String>>>,
    track_submit: Signal<Option<Result<(), String>>>,
    password: ReadSignal<String>,
    meeting_id: ReadSignal<String>,
    reviews: Signal<Option<Result<Reviews, String>>>,
    tracks: Signal<Option<Result<Vec<AlbumTrack>, String>>>,
    track_ratings: Signal<HashMap<String, u8>>,
) -> Element {
    rsx! {
        div { class: "review-form-container",
            div { class: "review-logged-banner",
                span { "Inloggad som " }
                strong { "{logged_name}" }
                button {
                    class: "review-logout-btn",
                    onclick: move |_| {
                        logged_in_as.set(None);
                        album_submit.set(None);
                        track_submit.set(None);
                    },
                    "Logga ut"
                }
            }

            // ── Album review ────────────────────────────
            div { class: "card review-section",
                h3 { "Albumbetyg" }
                p { class: "review-section-hint", "Ge albumet ett betyg från 0 till 10." }
                div { class: "review-star-row",
                    StarRating {
                        score: album_rating(),
                        on_change: move |s| {
                            album_rating.set(s);
                            album_submit.set(None);

                            let name = logged_in_as().unwrap_or_default();
                            let pw = password();
                            let mid = meeting_id();

                            spawn(async move {
                                let result = submit_album_review(name, pw, mid.clone(), s)
                                    .await
                                    .map_err(|e| e.to_string());
                                if result.is_ok() {
                                    if let Ok(fresh) = get_reviews(mid).await {
                                        reviews.set(Some(Ok(fresh)));
                                    }
                                }
                                album_submit.set(Some(result));

                            });
                        },
                    }
                    span { class: "review-score-text", "{album_rating()} / 10" }
                }

                if let Some(Err(ref e)) = album_submit() {
                    p { class: "review-error", "Fel: {e}" }
                }
            }

            // ── Track reviews ────────────────────────────
            match tracks() {
                None => rsx! {
                    div { class: "review-tracks-loading", "Laddar låtar…" }
                },
                Some(Err(ref e)) => rsx! {
                    div { class: "review-error", "Kunde inte ladda låtar: {e}" }
                },
                Some(Ok(ref track_list)) if track_list.is_empty() => rsx! {},
                Some(Ok(ref track_list)) => rsx! {
                    div { class: "card review-section",
                        h3 { "Låtbetyg" }
                        p { class: "review-section-hint", "Sätt ett betyg för varje låt." }

                        div { class: "review-track-list",
                            for track in track_list.iter() {
                                TrackRatingRow {
                                    key: "{track.track_id}",
                                    track: track.clone(),
                                    score: track_ratings().get(&track.track_id).copied().unwrap_or(0),
                                    on_change: {
                                        let tid = track.track_id.clone();
                                        move |s: u8| {
                                            track_ratings.write().insert(tid.clone(), s);
                                            track_submit.set(None);
                                        }
                                    },
                                }
                            }
                        }

                        if let Some(Err(ref e)) = track_submit() {
                            p { class: "review-error", "Fel: {e}" }
                        }
                    }
                },
            }
        }
    }
}
