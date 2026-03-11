use api::admin_delete_history_entry;
use api::admin_reorder_members;
use api::admin_set_current;
use api::admin_set_member_password;
use api::admin_spotify_album_search;
use api::admin_update_current;
use api::api_models::{Data, HistoryEntry, SetCurrentRequest, SpotifyAlbumSearchItem};
use api::{get_current, get_history};
use dioxus::document::eval;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fi_icons::FiTrash2;
use dioxus_free_icons::Icon;
use std::time::Duration;

const ADMIN_SCSS: Asset = asset!("/assets/styling/admin.scss");

// ── Shell layout ──────────────────────────────────────────────────────────────

/// Outer shell for all admin pages.
///
/// Initialises shared state, provides it via `AdminCtx`, and renders the
/// header, token input, tab bar, and then `children` (the active tab content).
///
/// `active_tab` must be one of `"album"`, `"rotation"`, `"historik"`, or
/// `"lossenord"` so the correct tab can be highlighted.
#[component]
pub fn AdminShell(active_tab: &'static str, children: Element) -> Element {
    let mut admin_token = use_signal(String::new);
    let members = use_signal(Vec::<String>::new);
    let original_members = use_signal(Vec::<String>::new);
    let current_data = use_signal(|| None::<Data>);
    let history = use_signal(|| None::<Result<Vec<HistoryEntry>, String>>);

    use_context_provider(|| AdminCtx {
        admin_token,
        members,
        original_members,
        current_data,
        history,
    });

    let mut members_w = members;
    let mut original_members_w = original_members;
    let mut current_data_w = current_data;
    let mut history_w = history;

    use_future(move || async move {
        if let Ok(data) = get_current().await {
            let list: Vec<String> = data.members.iter().map(|m| m.to_string()).collect();
            members_w.set(list.clone());
            original_members_w.set(list);
            current_data_w.set(Some(data));
        }
    });

    use_future(move || async move {
        let result = get_history().await.map_err(|e| e.to_string());
        history_w.set(Some(result));
    });

    let tab = move |slug: &'static str, href: &'static str, label: &'static str| {
        let class = if active_tab == slug {
            "admin-tab admin-tab-active"
        } else {
            "admin-tab"
        };
        rsx! {
            a { class, href, "{label}" }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: ADMIN_SCSS }

        div { class: "admin-page-wrapper",
            header {
                h1 { "Admin" }
                p { "Hantera nuvarande album, möte och väljare." }
            }

            div { class: "card admin-section",
                label { class: "admin-label", r#for: "admin-token",
                    "Admin-token"
                    span { class: "required-star", " *" }
                }
                input {
                    id: "admin-token",
                    r#type: "password",
                    placeholder: "ADMIN_TOKEN",
                    value: "{admin_token}",
                    oninput: move |e| admin_token.write().clone_from(&e.value()),
                }
            }

            div { class: "admin-tab-bar",
                {tab("album",    "/admin",          "Nytt album")}
                {tab("rotation", "/admin/rotation",  "Rotation")}
                {tab("historik", "/admin/historik",  "Historik")}
                {tab("lossenord", "/admin/l%C3%B6senord", "Lösenord")}
            }

            {children}

            crate::SiteFooter {}
        }
    }
}

async fn wait_for_debounce() {
    gloo_timers::future::sleep(Duration::from_millis(250)).await;
}

// ── Shared context ────────────────────────────────────────────────────────────

/// State shared across all admin tab components via the context API.
/// Provided by the `AdminLayout` component in the web crate.
#[derive(Clone, Copy)]
pub struct AdminCtx {
    pub admin_token: Signal<String>,
    pub members: Signal<Vec<String>>,
    pub original_members: Signal<Vec<String>>,
    pub current_data: Signal<Option<Data>>,
    pub history: Signal<Option<Result<Vec<HistoryEntry>, String>>>,
}

// ── Tab: Nytt album ───────────────────────────────────────────────────────────

#[component]
pub fn AdminAlbum() -> Element {
    let ctx = use_context::<AdminCtx>();
    let admin_token = ctx.admin_token;
    let members = ctx.members;
    let mut current_data = ctx.current_data;
    let mut history = ctx.history;

    let mut spotify_query = use_signal(String::new);
    let mut spotify_search_state =
        use_signal(|| None::<Result<Vec<SpotifyAlbumSearchItem>, String>>);
    let mut spotify_search_request_id = use_signal(|| 0_u64);

    let mut selected_album = use_signal(|| None::<SpotifyAlbumSearchItem>);
    let mut picker = use_signal(String::new);
    let mut meeting_date = use_signal(String::new);
    let mut meeting_time_val = use_signal(String::new);
    let mut meeting_location = use_signal(String::new);
    let mut submit_state = use_signal(|| None::<Result<(), String>>);
    let mut is_editing_current = use_signal(|| false);

    rsx! {
        if let Some(data) = current_data() {
            if data.current_album.is_some() {
                div { class: "card admin-section admin-edit-current-section",
                    if is_editing_current() {
                        div { class: "admin-editing-banner",
                            span { "Redigerar nuvarande album" }
                            button {
                                class: "admin-button-ghost",
                                onclick: move |_| {
                                    is_editing_current.set(false);
                                    selected_album.set(None);
                                    picker.set(String::new());
                                    meeting_date.set(String::new());
                                    meeting_time_val.set(String::new());
                                    meeting_location.set(String::new());
                                    spotify_query.set(String::new());
                                    spotify_search_state.set(None);
                                    submit_state.set(None);
                                },
                                "Avbryt"
                            }
                        }
                    } else {
                        button {
                            class: "admin-button",
                            onclick: move |_| {
                                if let Some(data) = current_data() {
                                    if let Some(album) = &data.current_album {
                                        selected_album.set(Some(SpotifyAlbumSearchItem {
                                            id: album.id.clone(),
                                            name: album.name.clone(),
                                            artists: album.artist.clone(),
                                            image_url: if album.album_art.is_empty() {
                                                None
                                            } else {
                                                Some(album.album_art.clone())
                                            },
                                            spotify_url: album.spotify_url.clone(),
                                        }));
                                    }
                                    if let Some(person) = &data.current_person {
                                        picker.set(person.to_string());
                                    }
                                    if let Some(meeting) = &data.next_meeting {
                                        meeting_date.set(meeting.date.clone());
                                        meeting_time_val.set(meeting.time.clone().unwrap_or_default());
                                        meeting_location
                                            .set(meeting.location.clone().unwrap_or_default());
                                    }
                                }
                                is_editing_current.set(true);
                                submit_state.set(None);
                            },
                            "Redigera nuvarande"
                        }
                    }
                }
            }
        }

        div { class: "card admin-section",
            h2 {
                "Välj album"
                span { class: "required-star", " *" }
            }

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
                            spotify_search_state
                                .set(Some(Err("Ange admin-token först".to_string())));
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
                                            p { class: "album-result-name", "{album.name}" }
                                            p { class: "album-result-artists", "{album.artists}" }
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

        div { class: "card admin-section",
            h2 { "Mötesinformation" }

            div { class: "admin-field-group",
                div { class: "admin-field",
                    label { class: "admin-label", r#for: "meeting-date",
                        "Datum"
                        span { class: "required-star", " *" }
                    }
                    input {
                        id: "meeting-date",
                        r#type: "date",
                        required: true,
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
                    label { class: "admin-label", r#for: "meeting-location", "Plats" }
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

        div { class: "card admin-section",
            h2 {
                "Väljare"
                span { class: "required-star", " *" }
            }
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

        div { class: "card admin-section admin-submit-section",
            p { class: "admin-required-note",
                span { class: "required-star", "*" }
                " Obligatoriska fält"
            }
            button {
                class: "admin-button admin-button-submit",
                disabled: selected_album().is_none()
                    || picker().is_empty()
                    || meeting_date().is_empty()
                    || admin_token().trim().is_empty(),
                onclick: move |_| {
                    let token = admin_token();
                    let Some(album) = selected_album() else { return; };
                    let picker_val = picker();
                    if picker_val.is_empty() { return; }

                    let opt_str = |s: String| -> Option<String> {
                        if s.trim().is_empty() { None } else { Some(s) }
                    };
                    let date = meeting_date();
                    let time = opt_str(meeting_time_val());
                    let location = opt_str(meeting_location());
                    let art_url = album.image_url.unwrap_or_default();
                    let editing = is_editing_current();

                    submit_state.set(None);
                    spawn(async move {
                        let req = SetCurrentRequest {
                            album_id: album.id,
                            album_name: album.name,
                            album_artist: album.artists,
                            album_art_url: art_url,
                            album_spotify_url: album.spotify_url,
                            picker: picker_val,
                            meeting_date: date,
                            meeting_time: time,
                            meeting_location: location,
                        };
                        let result = if editing {
                            admin_update_current(token, req).await.map_err(|e| e.to_string())
                        } else {
                            admin_set_current(token, req).await.map_err(|e| e.to_string())
                        };
                        if result.is_ok() {
                            if let Ok(fresh_data) = get_current().await {
                                current_data.set(Some(fresh_data));
                            }
                            if !editing {
                                let fresh = get_history().await.map_err(|e| e.to_string());
                                history.set(Some(fresh));
                            }
                            is_editing_current.set(false);
                            selected_album.set(None);
                            picker.set(String::new());
                            meeting_date.set(String::new());
                            meeting_time_val.set(String::new());
                            meeting_location.set(String::new());
                            spotify_query.set(String::new());
                            spotify_search_state.set(None);
                        }
                        submit_state.set(Some(result));
                    });
                },
                if is_editing_current() { "Uppdatera" } else { "Spara" }
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
}

// ── Tab: Rotation ─────────────────────────────────────────────────────────────

#[component]
pub fn AdminRotation() -> Element {
    let ctx = use_context::<AdminCtx>();
    let admin_token = ctx.admin_token;
    let mut members = ctx.members;
    let mut original_members = ctx.original_members;

    let mut reorder_state = use_signal(|| None::<Result<(), String>>);

    rsx! {
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
                                        members.write().clone_from(&list);
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
                                        members.write().clone_from(&list);
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
                disabled: members() == original_members() || admin_token().trim().is_empty(),
                onclick: move |_| {
                    let token = admin_token();
                    let ordered = members();
                    reorder_state.set(None);
                    spawn(async move {
                        let result = admin_reorder_members(token, ordered.clone())
                            .await
                            .map_err(|e| e.to_string());
                        if result.is_ok() {
                            original_members.write().clone_from(&ordered);
                        }
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
}

// ── Tab: Historik ─────────────────────────────────────────────────────────────

#[component]
pub fn AdminHistory() -> Element {
    let ctx = use_context::<AdminCtx>();
    let admin_token = ctx.admin_token;
    let mut history = ctx.history;

    // Refresh history every time this tab is mounted.
    use_effect(move || {
        history.set(None);
        spawn(async move {
            let result = get_history().await.map_err(|e| e.to_string());
            history.set(Some(result));
        });
    });

    rsx! {
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
                        for entry in {
                            let mut sorted = list.clone();
                            sorted.sort_unstable_by(|a, b| b.meeting_date.cmp(&a.meeting_date));
                            sorted
                        } {
                            div { class: "admin-history-row",
                                div { class: "admin-history-info",
                                    span { class: "admin-history-album", "{entry.album_name}" }
                                    span { class: "admin-history-meta",
                                        "{entry.album_artist} \u{2022} {entry.picker}"
                                    }
                                }
                                button {
                                    class: "admin-button-ghost admin-history-delete",
                                    title: "Ta bort",
                                    disabled: admin_token().trim().is_empty(),
                                    onclick: {
                                        let id = entry.id.clone();
                                        move |_| {
                                            let token = admin_token();
                                            let entry_id = id.clone();
                                            spawn(async move {
                                                if admin_delete_history_entry(
                                                    token,
                                                    entry_id.clone(),
                                                )
                                                .await
                                                .is_ok()
                                                {
                                                    if let Some(Ok(ref mut list)) =
                                                        *history.write()
                                                    {
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
}

// ── Tab: Lösenord ─────────────────────────────────────────────────────────────

#[component]
pub fn AdminPasswords() -> Element {
    let ctx = use_context::<AdminCtx>();
    let admin_token = ctx.admin_token;
    let members = ctx.members;

    let mut pw_member = use_signal(String::new);
    let mut pw_result = use_signal(|| None::<Result<String, String>>);
    let mut pw_copied = use_signal(|| false);

    rsx! {
        div { class: "card admin-section",
            h2 { "Generera lösenord" }
            p { class: "admin-hint",
                "Generera ett slumpmässigt lösenord för en medlem. \
                 Lösenordet visas en gång – dela det med medlemmen direkt."
            }

            div { class: "admin-field",
                label { class: "admin-label", r#for: "pw-member", "Medlem" }
                select {
                    id: "pw-member",
                    value: "{pw_member}",
                    onchange: move |e| {
                        pw_member.set(e.value());
                        pw_result.set(None);
                    },
                    option {
                        value: "",
                        disabled: true,
                        selected: pw_member().is_empty(),
                        "Välj medlem..."
                    }
                    for member in members() {
                        option {
                            value: "{member}",
                            selected: pw_member() == member,
                            "{member}"
                        }
                    }
                }
            }

            button {
                class: "admin-button admin-button-submit",
                disabled: pw_member().is_empty() || admin_token().trim().is_empty(),
                onclick: move |_| {
                    let token = admin_token();
                    let name = pw_member();
                    pw_result.set(None);
                    spawn(async move {
                        let result = admin_set_member_password(token, name)
                            .await
                            .map_err(|e| e.to_string());
                        pw_result.set(Some(result));
                    });
                },
                "Generera lösenord"
            }

            if let Some(Err(ref e)) = pw_result() {
                p { class: "admin-error", "Fel: {e}" }
            }
        }

        if let Some(Ok(ref plain)) = pw_result() {
            div {
                class: "admin-pw-modal-backdrop",
                onclick: move |_| {
                    pw_result.set(None);
                    pw_copied.set(false);
                },
                div {
                    class: "admin-pw-modal",
                    onclick: move |e| e.stop_propagation(),

                    div { class: "admin-pw-modal-header",
                        h2 { "Lösenord genererat" }
                        p { class: "admin-pw-modal-member",
                            "för "
                            strong { "{pw_member()}" }
                        }
                    }

                    div { class: "admin-pw-modal-code-wrap",
                        code { class: "admin-pw-code", "{plain}" }
                        button {
                            class: if pw_copied() {
                                "admin-button admin-pw-copy-btn admin-pw-copy-btn--done"
                            } else {
                                "admin-button admin-pw-copy-btn"
                            },
                            onclick: {
                                let text = plain.clone();
                                move |_| {
                                    let text = text.clone();
                                    spawn(async move {
                                        let _ = eval(
                                            &format!("navigator.clipboard.writeText('{text}')"),
                                        )
                                        .await;
                                        pw_copied.set(true);
                                    });
                                }
                            },
                            if pw_copied() { "✓ Kopierat" } else { "Kopiera" }
                        }
                    }

                    div { class: "admin-pw-modal-warning",
                        span { class: "admin-pw-warning-icon", "⚠" }
                        p {
                            "Det här lösenordet visas bara en gång och "
                            strong { "kan inte återskapas" }
                            ". Dela det med medlemmen innan du stänger."
                        }
                    }

                    button {
                        class: "admin-button admin-pw-modal-close",
                        onclick: move |_| {
                            pw_result.set(None);
                            pw_copied.set(false);
                        },
                        "Stäng"
                    }
                }
            }
        }
    }
}
