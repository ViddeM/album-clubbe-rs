use api::admin_delete_history_entry;
use api::admin_reorder_members;
use api::admin_set_current;
use api::admin_spotify_album_search;
use api::api_models::{HistoryEntry, SpotifyAlbumSearchItem};
use api::{get_current, get_history};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fi_icons::FiTrash2;
use dioxus_free_icons::Icon;
use std::time::Duration;

const ADMIN_SCSS: Asset = asset!("/assets/styling/admin.scss");

async fn wait_for_debounce() {
    gloo_timers::future::sleep(Duration::from_millis(250)).await;
}

#[derive(Clone, PartialEq)]
enum AdminTab {
    Album,
    Rotation,
    History,
}

#[component]
pub fn Admin() -> Element {
    let mut admin_token = use_signal(String::new);
    let mut active_tab = use_signal(|| AdminTab::Album);

    // Spotify search
    let mut spotify_query = use_signal(String::new);
    let mut spotify_search_state =
        use_signal(|| None::<Result<Vec<SpotifyAlbumSearchItem>, String>>);
    let mut spotify_search_request_id = use_signal(|| 0_u64);

    // Form state
    let mut selected_album = use_signal(|| None::<SpotifyAlbumSearchItem>);
    let mut picker = use_signal(String::new);
    let mut meeting_date = use_signal(String::new);
    let mut meeting_time_val = use_signal(String::new);
    let mut meeting_location = use_signal(String::new);

    // Members + submit feedback
    let mut members = use_signal(Vec::<String>::new);
    let mut submit_state = use_signal(|| None::<Result<(), String>>);
    let mut reorder_state = use_signal(|| None::<Result<(), String>>);
    let mut history = use_signal(|| None::<Result<Vec<HistoryEntry>, String>>);

    // Load members on mount via the info endpoint.
    use_future(move || async move {
        if let Ok(data) = get_current().await {
            members.set(data.members.iter().map(|m| m.to_string()).collect());
        }
    });

    // Load history on mount.
    use_future(move || async move {
        let result = get_history().await.map_err(|e| e.to_string());
        history.set(Some(result));
    });

    rsx! {
        document::Link { rel: "stylesheet", href: ADMIN_SCSS }
        div { class: "admin-page-wrapper",
            header {
                h1 { "Admin" }
                p { "Hantera nuvarande album, möte och väljare." }
            }

            // ── Token ───────────────────────────────────────────────────────
            div { class: "card admin-section",
                label { class: "admin-label", r#for: "admin-token", "Admin-token" }
                input {
                    id: "admin-token",
                    r#type: "password",
                    placeholder: "ADMIN_TOKEN",
                    value: "{admin_token}",
                    oninput: move |e| admin_token.set(e.value()),
                }
            }

            // ── Tab bar ─────────────────────────────────────────────────────
            div { class: "admin-tab-bar",
                button {
                    class: if active_tab() == AdminTab::Album { "admin-tab admin-tab-active" } else { "admin-tab" },
                    onclick: move |_| active_tab.set(AdminTab::Album),
                    "Nytt album"
                }
                button {
                    class: if active_tab() == AdminTab::Rotation { "admin-tab admin-tab-active" } else { "admin-tab" },
                    onclick: move |_| active_tab.set(AdminTab::Rotation),
                    "Rotation"
                }
                button {
                    class: if active_tab() == AdminTab::History { "admin-tab admin-tab-active" } else { "admin-tab" },
                    onclick: move |_| {
                        active_tab.set(AdminTab::History);
                        history.set(None);
                        spawn(async move {
                            let result = get_history().await.map_err(|e| e.to_string());
                            history.set(Some(result));
                        });
                    },
                    "Historik"
                }
            }

            // ── Tab: Nytt album ─────────────────────────────────────────────
            if active_tab() == AdminTab::Album {
                // ── Album search ─────────────────────────────────────────────
                div { class: "card admin-section",
                    h2 { "Välj album" }

                    if let Some(album) = selected_album() {
                        div { class: "album-result-card selected-album-card",
                            if let Some(ref image_url) = album.image_url {
                                img {
                                    class: "album-result-thumb",
                                    src: "{image_url}",
                                    alt: "{album.name}",
                                }
                            }
                            div { class: "album-result-info",
                                p { class: "album-result-name", "{album.name}" }
                                p { class: "album-result-artists", "{album.artists}" }
                                span { class: "admin-badge", "Valt" }
                            }
                            button {
                                class: "admin-button-ghost",
                                onclick: move |_| selected_album.set(None),
                                "Ändra"
                            }
                        }
                    } else {
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

                        if let Some(state) = spotify_search_state() {
                            if let Ok(albums) = state {
                                if !albums.is_empty() {
                                    div { class: "album-search-results",
                                        for album in albums {
                                            div {
                                                class: "album-result-card",
                                                key: "{album.id}",
                                                if let Some(ref image_url) = album.image_url {
                                                    img {
                                                        class: "album-result-thumb",
                                                        src: "{image_url}",
                                                        alt: "{album.name}",
                                                    }
                                                }
                                                div { class: "album-result-info",
                                                    p { class: "album-result-name",
                                                        "{album.name}"
                                                    }
                                                    p { class: "album-result-artists",
                                                        "{album.artists}"
                                                    }
                                                }
                                                button {
                                                    class: "admin-button",
                                                    onclick: {
                                                        let album = album.clone();
                                                        move |_| {
                                                            selected_album.set(Some(album.clone()));
                                                            spotify_search_state.set(None);
                                                            spotify_query.set(String::new());
                                                        }
                                                    },
                                                    "Välj"
                                                }
                                            }
                                        }
                                    }
                                } else if !spotify_query().is_empty() {
                                    p { "Inga träffar" }
                                }
                            } else if let Err(err) = state {
                                p { class: "admin-error", "Fel: {err}" }
                            }
                        }
                    }
                }

                // ── Meeting details ──────────────────────────────────────────
                div { class: "card admin-section",
                    h2 { "Mötesinformation" }

                    div { class: "admin-field-group",
                        div { class: "admin-field",
                            label { class: "admin-label", r#for: "meeting-date", "Datum" }
                            input {
                                id: "meeting-date",
                                r#type: "date",
                                value: "{meeting_date}",
                                oninput: move |e| {
                                    let val = e.value();
                                    if val.is_empty() {
                                        meeting_time_val.set(String::new());
                                    }
                                    meeting_date.set(val);
                                },
                            }
                        }
                        div { class: "admin-field",
                            label { class: "admin-label", r#for: "meeting-time", "Tid" }
                            input {
                                id: "meeting-time",
                                r#type: "time",
                                lang: "sv",
                                disabled: meeting_date().is_empty(),
                                value: "{meeting_time_val}",
                                oninput: move |e| meeting_time_val.set(e.value()),
                            }
                        }
                        div { class: "admin-field",
                            label {
                                class: "admin-label",
                                r#for: "meeting-location",
                                "Plats"
                            }
                            input {
                                id: "meeting-location",
                                r#type: "text",
                                placeholder: "t.ex. Discord",
                                value: "{meeting_location}",
                                oninput: move |e| meeting_location.set(e.value()),
                            }
                        }
                    }
                }

                // ── Picker ───────────────────────────────────────────────────
                div { class: "card admin-section",
                    h2 { "Väljare" }
                    select {
                        value: "{picker}",
                        onchange: move |e| picker.set(e.value()),
                        option {
                            value: "",
                            disabled: true,
                            selected: picker().is_empty(),
                            "Välj person..."
                        }
                        for member in members() {
                            option {
                                value: "{member}",
                                selected: picker() == member,
                                "{member}"
                            }
                        }
                    }
                }

                // ── Submit ───────────────────────────────────────────────────
                div { class: "card admin-section admin-submit-section",
                    button {
                        class: "admin-button admin-button-submit",
                        disabled: selected_album().is_none() || picker().is_empty(),
                        onclick: move |_| {
                            let token = admin_token();
                            let Some(album) = selected_album() else {
                                return;
                            };
                            let picker_val = picker();
                            if picker_val.is_empty() {
                                return;
                            }

                            let opt_str = |s: String| -> Option<String> {
                                if s.trim().is_empty() { None } else { Some(s) }
                            };
                            let date = opt_str(meeting_date());
                            let time = opt_str(meeting_time_val());
                            let location = opt_str(meeting_location());
                            let art_url = album.image_url.unwrap_or_default();

                            submit_state.set(None);
                            spawn(async move {
                                let result = admin_set_current(
                                        token,
                                        album.id,
                                        album.name,
                                        album.artists,
                                        art_url,
                                        album.spotify_url,
                                        picker_val,
                                        date,
                                        time,
                                        location,
                                    )
                                    .await
                                    .map_err(|e| e.to_string());
                                if result.is_ok() {
                                    let fresh = get_history().await.map_err(|e| e.to_string());
                                    history.set(Some(fresh));
                                }
                                submit_state.set(Some(result));
                            });
                        },
                        "Spara"
                    }

                    if let Some(result) = submit_state() {
                        if result.is_ok() {
                            p { class: "admin-success", "✓ Sparat!" }
                        } else if let Err(err) = result {
                            p { class: "admin-error", "Fel: {err}" }
                        }
                    }
                }
            }

            // ── Tab: Rotation ───────────────────────────────────────────────
            if active_tab() == AdminTab::Rotation {
                div { class: "card admin-section",
                    h2 { "Medlemsordning" }
                    p { class: "admin-hint", "Ändra ordningen på medlemmarna i rotationen." }

                    div { class: "member-order-list",
                        for (i , member) in members().iter().enumerate() {
                            div { key: "{member}", class: "member-order-row",
                                span { class: "member-order-name", "{member}" }
                                div { class: "member-order-buttons",
                                    button {
                                        class: "admin-button-ghost",
                                        disabled: i == 0,
                                        onclick: move |_| {
                                            let mut list = members();
                                            if i > 0 {
                                                list.swap(i - 1, i);
                                                members.set(list);
                                                reorder_state.set(None);
                                            }
                                        },
                                        "↑"
                                    }
                                    button {
                                        class: "admin-button-ghost",
                                        disabled: i + 1 >= members().len(),
                                        onclick: move |_| {
                                            let mut list = members();
                                            if i + 1 < list.len() {
                                                list.swap(i, i + 1);
                                                members.set(list);
                                                reorder_state.set(None);
                                            }
                                        },
                                        "↓"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: "admin-button admin-button-submit",
                        onclick: move |_| {
                            let token = admin_token();
                            let ordered = members();
                            reorder_state.set(None);
                            spawn(async move {
                                let result = admin_reorder_members(token, ordered)
                                    .await
                                    .map_err(|e| e.to_string());
                                reorder_state.set(Some(result));
                            });
                        },
                        "Spara ordning"
                    }

                    if let Some(result) = reorder_state() {
                        if result.is_ok() {
                            p { class: "admin-success", "✓ Ordning sparad!" }
                        } else if let Err(err) = result {
                            p { class: "admin-error", "Fel: {err}" }
                        }
                    }
                }
            }

            // ── Tab: Historik ───────────────────────────────────────────────
            if active_tab() == AdminTab::History {
                div { class: "card admin-section",
                    h2 { "Historik" }
                    p { class: "admin-hint", "Ta bort tidigare poster." }

                    match history() {
                        None => rsx! {
                            p { class: "admin-hint", "Laddar\u{2026}" }
                        },
                        Some(Err(e)) => rsx! {
                            p { class: "admin-error", "Fel: {e}" }
                        },
                        Some(Ok(list)) if list.is_empty() => rsx! {
                            p { class: "admin-hint", "Inga tidigare poster." }
                        },
                        Some(Ok(list)) => rsx! {
                            div { class: "admin-history-list",
                                for entry in list {
                                    div { class: "admin-history-row",
                                        div { class: "admin-history-info",
                                            span { class: "admin-history-album", "{entry.album_name}" }
                                            span { class: "admin-history-meta", "{entry.album_artist} \u{2022} {entry.picker}" }
                                        }
                                        button {
                                            class: "admin-button-ghost admin-history-delete",
                                            title: "Ta bort",
                                            onclick: {
                                                let id = entry.id.clone();
                                                move |_| {
                                                    let token = admin_token();
                                                    let entry_id = id.clone();
                                                    spawn(async move {
                                                        if admin_delete_history_entry(token, entry_id.clone()).await.is_ok() {
                                                            if let Some(Ok(ref mut list)) = *history.write() {
                                                                list.retain(|e| e.id != entry_id);
                                                            }
                                                        }
                                                    });
                                                }
                                            },
                                            Icon { icon: FiTrash2 }
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }

            footer { class: "site-footer",
                a { href: "/", "Tillbaka till startsidan" }
            }
        }
    }
}
