use dioxus::prelude::*;

mod admin_view;
mod history_view;
mod main_view;
mod review_view;
pub use admin_view::Admin;
pub use history_view::History;
pub use main_view::Main;
pub use review_view::Review;

const GLOBAL_SCSS: Asset = asset!("/assets/styling/globals.scss");

#[component]
pub fn Setup() -> Element {
    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1" }
        document::Link { rel: "stylesheet", href: GLOBAL_SCSS }
    }
}

/// Shared navigation footer rendered on every page.
#[component]
pub fn SiteFooter() -> Element {
    rsx! {
        footer { class: "site-footer",
            a { href: "/", "Startsida" }
            a { href: "/history", "Historik" }
            a { href: "/review", "Granska" }
            a { href: "/admin", "Admin" }
        }
    }
}
