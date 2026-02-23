use dioxus::prelude::*;

use ui::{Admin as AdminView, History as HistoryView, Main, Setup};

fn main() {
    #[cfg(feature = "server")]
    {
        dioxus::prelude::serve(|| async move {
            use dioxus::prelude::{dioxus_server::axum, DioxusRouterExt, ServeConfig};

            dotenvy::dotenv().ok();

            Ok(axum::Router::new().serve_dioxus_application(ServeConfig::new(), App))
        });
    }

    #[cfg(not(feature = "server"))]
    {
        dioxus::launch(App);
    }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/history")]
    HistoryPage {},
    #[route("/admin")]
    Admin {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Albumklubben" }

        Setup {}
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Main {}
    }
}

#[component]
fn HistoryPage() -> Element {
    rsx! {
        HistoryView {}
    }
}

#[component]
fn Admin() -> Element {
    rsx! {
        AdminView {}
    }
}
