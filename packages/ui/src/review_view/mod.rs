mod aggregate_scores;
mod album_overview;
mod logged_in;
mod login;

use crate::review_view::aggregate_scores::AggregateScores;
use crate::review_view::album_overview::CurrentAlbumView;
use crate::review_view::logged_in::ReviewLoggedInView;
use crate::review_view::login::ReviewLoginView;
use crate::SiteFooter;
use api::api_models::{Album, AlbumTrack, Data, Name, Reviews};
use api::{
    get_album_tracks, get_current, get_reviews, submit_album_review, submit_track_review,
    verify_member,
};
use dioxus::prelude::*;

const REVIEW_SCSS: Asset = asset!("/assets/styling/review.scss");

#[component]
pub fn Review() -> Element {
    let mut page_data: Signal<Option<Data>> = use_signal(|| None);

    // Error
    let mut initial_load_error: Signal<Option<String>> = use_signal(|| None);

    let load_data = use_callback(move |()| {
        spawn(async move {
            let data_response = get_current().await;

            match data_response {
                Ok(data) => page_data.set(Some(data)),
                Err(err) => initial_load_error.set(Some(err.to_string())),
            }
        });
    });

    // Load data on mount
    use_future(move || async move { load_data(()) });

    rsx! {
        document::Link { rel: "stylesheet", href: REVIEW_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Recensera" }
            }

            if let Some(err) = initial_load_error() {
                div { class: "review-error", "Kunde inte ladda data: {err}" }
            } else if let Some(data) = page_data() {
                DisplayData { data }
            } else {
                div { class: "review-loading", "Laddar…" }
            }

            SiteFooter {}
        }
    }
}

#[component]
fn DisplayData(data: ReadSignal<Data>) -> Element {
    let Some(meeting_id) = data().current_meeting_id else {
        return rsx! {
            div { class: "review-empty card",
                p { "Inget möte inplanerat" }
                p {
                    a { href: "/", "Gå till startsidan" }
                }
            }
        };
    };

    let Some(album) = data().current_album else {
        return rsx! {
            div { class: "review-empty card",
                p { "Inget album att recensera just nu." }
                p {
                    a { href: "/", "Gå till startsidan" }
                }
            }
        };
    };

    let Some(current_person) = data().current_person else {
        return rsx! {
            div { class: "review-empty card",
                p { "Ingen nuvarande person" }
                p {
                    a { href: "/", "Gå till startsidan" }
                }
            }
        };
    };

    let members = use_memo(move || data().members);

    rsx! {
        ReviewAlbumView {
            meeting_id,
            album,
            members,
            current_person,
        }
    }
}

#[component]
fn ReviewAlbumView(
    meeting_id: ReadSignal<String>,
    album: ReadSignal<Album>,
    members: ReadSignal<Vec<Name>>,
    current_person: ReadSignal<Name>,
) -> Element {
    let mut tracks: Signal<Option<Vec<AlbumTrack>>> = use_signal(|| None);
    let mut reviews: Signal<Option<Reviews>> = use_signal(|| None);

    let mut load_error: Signal<Option<String>> = use_signal(|| None);

    let load_tracks = use_callback(move |()| {
        spawn(async move {
            let tracks_response = get_album_tracks(album().id).await;

            match tracks_response {
                Ok(ts) => {
                    tracks.set(Some(ts));
                }
                Err(err) => {
                    load_error.set(Some(err.to_string()));
                }
            }
        });
    });

    let update_reviews = use_callback(move |new_reviews: Reviews| {
        reviews.set(Some(new_reviews));
    });

    let handle_reviews_response =
        use_callback(move |reviews_response: Result<Reviews, ServerFnError>| {
            match reviews_response {
                Ok(r) => {
                    reviews.set(Some(r));
                }
                Err(err) => {
                    load_error.set(Some(err.to_string()));
                }
            };
        });

    let load_reviews = use_callback(move |()| {
        spawn(async move {
            let load_reviews_response = get_reviews(meeting_id()).await;
            handle_reviews_response(load_reviews_response);
        });
    });

    use_future(move || async move {
        load_tracks(());
        load_reviews(());
    });

    if let Some(error) = load_error() {
        return rsx! {
            div { class: "review-empty card",
                p { "Misslyckades med att ladda recensionsdata, {error}" }
                p {
                    a { href: "/", "Gå till startsidan" }
                }
            }
        };
    }

    let Some(tracks) = tracks() else {
        return rsx! {
            div { class: "review-loading", "Laddar…" }
        };
    };

    let Some(reviews) = reviews() else {
        return rsx! {
            div { class: "review-loading", "Laddar…" }
        };
    };

    rsx! {
        CurrentAlbumView { album, picked_by: current_person() }

        DisplayAndPerformReviewView {
            reviews,
            tracks,
            members,
            meeting_id,
            update_reviews,
        }
    }
}

#[component]
fn DisplayAndPerformReviewView(
    reviews: ReadSignal<Reviews>,
    tracks: ReadSignal<Vec<AlbumTrack>>,
    members: ReadSignal<Vec<Name>>,
    meeting_id: ReadSignal<String>,
    update_reviews: Callback<Reviews, ()>,
) -> Element {
    rsx! {
        AggregateScores { reviews, tracks }

        PerformReviewView {
            reviews,
            tracks,
            members,
            meeting_id,
            update_reviews,
        }
    }
}

#[component]
fn PerformReviewView(
    reviews: ReadSignal<Reviews>,
    tracks: ReadSignal<Vec<AlbumTrack>>,
    members: ReadSignal<Vec<Name>>,
    meeting_id: ReadSignal<String>,
    update_reviews: Callback<Reviews, ()>,
) -> Element {
    let mut member_name = use_signal(String::new);
    let mut password = use_signal(String::new);

    let mut logged_in_as = use_signal(|| None::<String>);
    let mut login_error = use_signal(|| None::<String>);

    let mut album_review_error: Signal<Option<String>> = use_signal(|| None);
    let mut track_review_error: Signal<Option<String>> = use_signal(|| None);

    let perform_login = use_callback(move |_: ()| {
        login_error.set(None);
        spawn(async move {
            match verify_member(member_name(), password()).await {
                Ok(()) => {
                    login_error.set(None);
                    logged_in_as.set(Some(member_name()));
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                }
            }
        });
    });

    let logout = use_callback(move |()| {
        logged_in_as.set(None);
        member_name.set(String::new());
        password.set(String::new());
    });

    let review_album = use_callback(move |review| {
        spawn(async move {
            let result = submit_album_review(member_name(), password(), meeting_id(), review).await;

            match result {
                Ok(r) => {
                    update_reviews(r);
                }
                Err(err) => {
                    album_review_error.set(Some(err.to_string()));
                }
            }
        });
    });

    let review_track = use_callback(move |(track_id, review)| {
        spawn(async move {
            let result =
                submit_track_review(member_name(), password(), meeting_id(), track_id, review)
                    .await;

            match result {
                Ok(r) => update_reviews(r),
                Err(err) => track_review_error.set(Some(err.to_string())),
            }
        });
    });

    let reset_errors = use_callback(move |()| {
        album_review_error.set(None);
        track_review_error.set(None);
    });

    let Some(logged_in_as) = logged_in_as() else {
        return rsx! {
            ReviewLoginView {
                member_name,
                password,
                login_error,
                members,
                perform_login,
            }
        };
    };

    rsx! {
        ReviewLoggedInView {
            logged_in_as,
            reviews,
            review_album,
            review_track,
            logout,
            album_review_error,
            track_review_error,
            reset_errors,
        }
    }
}
