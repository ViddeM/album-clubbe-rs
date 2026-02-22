use dioxus::prelude::*;

mod main_view;
pub use main_view::Main;

const GLOBAL_SCSS: Asset = asset!("/assets/styling/globals.scss");

#[component]
pub fn Setup() -> Element {
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Link { rel: "stylesheet", href: GLOBAL_SCSS }
    }
}
