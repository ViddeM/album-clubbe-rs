use dioxus::prelude::*;

const MAIN_SCSS: Asset = asset!("/assets/styling/main.scss");

#[component]
pub fn Main() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_SCSS }

        div { class: "page-wrapper",
            header {
                h1 { "Albumklubben" }
            }

            div { class: "first-row",
                div { class: "card", "asd" }
            }
        }
    }
}
