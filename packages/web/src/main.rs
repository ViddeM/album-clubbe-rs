use dioxus::prelude::*;

use ui::{Admin as AdminView, History as HistoryView, Main, Review as ReviewView, Setup};

fn main() {
    #[cfg(feature = "server")]
    {
        dioxus::prelude::serve(|| async move {
            use dioxus::prelude::{dioxus_server::axum, DioxusRouterExt, ServeConfig};

            dotenvy::dotenv().ok();

            let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
            tracing::info!("Starting server on port {port}");

            // Eagerly initialise the DB pool so startup errors surface immediately.
            if let Err(e) = api::init_db().await {
                tracing::error!("Database initialisation failed: {e}");
                std::process::exit(1);
            }

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
    #[route("/review")]
    ReviewPage {},
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
fn ReviewPage() -> Element {
    rsx! {
        ReviewView {}
    }
}

#[component]
fn Admin() -> Element {
    rsx! {
        AdminView {}
    }
}
