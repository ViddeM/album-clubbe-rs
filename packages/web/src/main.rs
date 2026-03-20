use dioxus::prelude::*;

use ui::{
    AdminAlbum, AdminHistory, AdminPasswords, AdminRotation, AdminShell,
    History as HistoryView, Main, Review as ReviewView, Setup,
};

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
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},
    #[route("/history")]
    HistoryPage {},
    #[route("/review")]
    ReviewPage {},
    #[layout(AdminLayout)]
        #[route("/admin")]
        AdminAlbumPage {},
        #[route("/admin/rotation")]
        AdminRotationPage {},
        #[route("/admin/historik")]
        AdminHistoryPage {},
        #[route("/admin/lösenord")]
        AdminPasswordsPage {},
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
    rsx! { Main {} }
}

#[component]
fn HistoryPage() -> Element {
    rsx! { HistoryView {} }
}

#[component]
fn ReviewPage() -> Element {
    rsx! { ReviewView {} }
}

/// Shared admin layout: wraps all admin routes in `AdminShell` which loads
/// data, provides `AdminCtx`, and renders the header, token input, and tab bar.
#[component]
fn AdminLayout() -> Element {
    let route: Route = use_route();
    let active_tab = if matches!(route, Route::AdminRotationPage {}) {
        "rotation"
    } else if matches!(route, Route::AdminHistoryPage {}) {
        "historik"
    } else if matches!(route, Route::AdminPasswordsPage {}) {
        "lossenord"
    } else {
        "album"
    };

    rsx! {
        AdminShell { active_tab,
            Outlet::<Route> {}
        }
    }
}

#[component]
fn AdminAlbumPage() -> Element {
    rsx! { AdminAlbum {} }
}

#[component]
fn AdminRotationPage() -> Element {
    rsx! { AdminRotation {} }
}

#[component]
fn AdminHistoryPage() -> Element {
    rsx! { AdminHistory {} }
}

#[component]
fn AdminPasswordsPage() -> Element {
    rsx! { AdminPasswords {} }
}
