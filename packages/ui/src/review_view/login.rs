use api::api_models::Name;
use dioxus::prelude::*;

#[component]
pub fn ReviewLoginView(
    member_name: Signal<String>,
    password: Signal<String>,
    login_error: Signal<Option<String>>,
    members: ReadSignal<Vec<Name>>,
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
                                perform_login(());
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
