use api::api_models::{AdminOverview, AdminPing, SpotifyAlbumSearchItem};
use api::{admin_ping, admin_spotify_album_search, get_admin_overview};
use dioxus::prelude::*;
use std::time::Duration;

async fn wait_for_debounce() {
    gloo_timers::future::sleep(Duration::from_millis(250)).await;
}

#[component]
pub fn Admin() -> Element {
    let mut admin_token = use_signal(String::new);
    let mut spotify_query = use_signal(String::new);
    let mut overview_state = use_signal(|| None::<Result<AdminOverview, String>>);
    let mut ping_state = use_signal(|| None::<Result<AdminPing, String>>);
    let mut spotify_search_state =
        use_signal(|| None::<Result<Vec<SpotifyAlbumSearchItem>, String>>);
    let mut spotify_search_request_id = use_signal(|| 0_u64);

    rsx! {
        div { class: "page-wrapper",
            header {
                h1 { "Admin" }
                p { "Ange admin-token för att köra skyddade endpoints." }
                a { href: "/", "Tillbaka till startsidan" }
            }

            div { class: "card",
                div { class: "gap-6",
                    input {
                        r#type: "password",
                        placeholder: "ADMIN_TOKEN",
                        value: "{admin_token}",
                        oninput: move |event| admin_token.set(event.value()),
                    }

                    div { class: "gap-6",
                        button {
                            onclick: move |_| {
                                let token = admin_token();
                                spawn(async move {
                                    let result = get_admin_overview(token)
                                        .await
                                        .map_err(|err| err.to_string());
                                    overview_state.set(Some(result));
                                });
                            },
                            "Hämta översikt"
                        }

                        button {
                            onclick: move |_| {
                                let token = admin_token();
                                spawn(async move {
                                    let result = admin_ping(token)
                                        .await
                                        .map_err(|err| err.to_string());
                                    ping_state.set(Some(result));
                                });
                            },
                            "Ping"
                        }
                    }
                }
            }

            div { class: "card",
                h2 { "Översikt" }
                if let Some(result) = overview_state() {
                    if let Ok(overview) = result {
                        p { "Medlemmar: {overview.members_count}" }
                        p { "Nuvarande väljare: {overview.current_picker}" }
                        p {
                            if overview.has_scheduled_meeting {
                                "Möte planerat: ja"
                            } else {
                                "Möte planerat: nej"
                            }
                        }
                    } else if let Err(err) = result {
                        p { "Fel: {err}" }
                    }
                } else {
                    p { "Ingen data ännu" }
                }
            }

            div { class: "card",
                h2 { "Ping" }
                if let Some(result) = ping_state() {
                    if let Ok(ping) = result {
                        p { "Status: {ping.status}" }
                    } else if let Err(err) = result {
                        p { "Fel: {err}" }
                    }
                } else {
                    p { "Ingen ping ännu" }
                }
            }

            div { class: "card",
                h2 { "Spotify albumsök" }

                input {
                    r#type: "text",
                    placeholder: "Sök album...",
                    value: "{spotify_query}",
                    oninput: move |event| {
                        let query = event.value();
                        spotify_query.set(query.clone());

                        spotify_search_request_id += 1;
                        let current_request_id = spotify_search_request_id();

                        if query.trim().is_empty() {
                            spotify_search_state.set(Some(Ok(Vec::new())));
                            return;
                        }

                        let token = admin_token();
                        if token.trim().is_empty() {
                            spotify_search_state.set(Some(Err("Ange admin-token först".to_string())));
                            return;
                        }

                        spawn(async move {
                            wait_for_debounce().await;

                            if spotify_search_request_id() != current_request_id {
                                return;
                            }

                            let result = admin_spotify_album_search(token, query)
                                .await
                                .map_err(|err| err.to_string());

                            if spotify_search_request_id() != current_request_id {
                                return;
                            }

                            spotify_search_state.set(Some(result));
                        });
                    },
                }

                if let Some(result) = spotify_search_state() {
                    if let Ok(albums) = result {
                        if albums.is_empty() {
                            p { "Inga träffar" }
                        } else {
                            for album in albums {
                                div { key: "{album.id}",

                                    if let Some(image_url) = album.image_url {
                                        img {
                                            src: "{image_url}",
                                            alt: "{album.name}",
                                            width: "64",
                                            height: "64",
                                        }
                                    }

                                    p { "{album.name}" }
                                    p { "{album.artists}" }
                                    a {
                                        href: "{album.spotify_url}",
                                        target: "_blank",
                                        rel: "noopener noreferrer",
                                        "Öppna i Spotify"
                                    }
                                }
                            }
                        }
                    } else if let Err(err) = result {
                        p { "Fel: {err}" }
                    }
                } else {
                    p { "Ingen sökning ännu" }
                }
            }
        }
    }
}
