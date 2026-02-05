use dioxus::prelude::*;

const MAIN_SCSS: Asset = asset!("/assets/styling/main.scss");
const MUSICAL_NOTE_SVG: Asset = asset!("/assets/icons/musical_note.svg");

#[component]
pub fn Main() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Albumklubben" }
            }

            div { class: "first-row row",
                div { class: "double-column",
                    div { class: "card",
                        div { class: "gap-2",
                            img {
                                src: MUSICAL_NOTE_SVG,
                                alt: "musical-note",
                                class: "note-icon",
                            }
                        }
                        div { class: "gap-2" }
                    }
                }

                div {
                    div { class: "card", "bsd" }
                }
            }

            div { class: "row",
                div { class: "card", "PEPE" }
            }
        }
    }
}
