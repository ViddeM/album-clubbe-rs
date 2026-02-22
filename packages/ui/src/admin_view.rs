use api::admin_spotify_album_search;
use api::api_models::SpotifyAlbumSearchItem;
use dioxus::prelude::*;
use std::time::Duration;

const ADMIN_SCSS: Asset = asset!("/assets/styling/admin.scss");

async fn wait_for_debounce() {
    gloo_timers::future::sleep(Duration::from_millis(250)).await;
}

#[component]
pub fn Admin() -> Element {
    let mut admin_token = use_signal(String::new);
    let mut spotify_query = use_signal(String::new);
    let mut spotify_search_state =
        use_signal(|| None::<Result<Vec<SpotifyAlbumSearchItem>, String>>);
    let mut spotify_search_request_id = use_signal(|| 0_u64);

    rsx! {
        document::Link { rel: "stylesheet", href: ADMIN_SCSS }
        div { class: "admin-page-wrapper",
            header {
                h1 { "Admin" }
                p { "Ange admin-token för att köra skyddade endpoints." }
                a { href: "/", "Tillbaka till startsidan" }
            }

            div { class: "card admin-search",
                h2 { "Spotify albumsök" }

                input {
                    r#type: "password",
                    placeholder: "ADMIN_TOKEN",
                    value: "{admin_token}",
                    oninput: move |event| admin_token.set(event.value()),
                }

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
                        if !albums.is_empty() {
                            div { class: "album-search-results",
                                for album in albums {
                                    div {
                                        class: "card album-result-card",
                                        key: "{album.id}",
                                        if let Some(image_url) = album.image_url {
                                            img {
                                                class: "album-result-thumb",
                                                src: "{image_url}",
                                                alt: "{album.name}",
                                            }
                                        }
                                        div { class: "album-result-info",
                                            p { class: "album-result-name", "{album.name}" }
                                            p { class: "album-result-artists", "{album.artists}" }
                                            a {
                                                class: "admin-spotify-link",
                                                href: "{album.spotify_url}",
                                                target: "_blank",
                                                rel: "noopener noreferrer",
                                                "Öppna i Spotify"
                                            }
                                        }
                                    }
                                }
                            }
                        } else if !spotify_query().is_empty() {
                            p { "Inga träffar" }
                        }
                    } else if let Err(err) = result {
                        p { "Fel: {err}" }
                    }
                }
            }
        }
    }
}
