use api::api_models::{Album, Name};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{fa_brands_icons::FaSpotify, fi_icons::FiExternalLink},
    Icon,
};

#[component]
pub fn CurrentAlbumView(album: ReadSignal<Album>, picked_by: ReadSignal<Option<Name>>) -> Element {
    rsx! {
        // ── Album info ──────────────────────────────────────
        div { class: "card review-album-card",
            div { class: "review-album-art-wrap",
                img {
                    class: "review-album-art",
                    src: "{album().album_art}",
                    alt: "{album().name} album cover",
                }
            }
            div { class: "review-album-info",
                h2 { class: "review-album-name", "{album().name}" }
                p { class: "review-album-artist", "{album().artist}" }
                if let Some(ref picker) = picked_by() {
                    p { class: "review-album-picker",
                        "Vald av "
                        span { class: "review-album-picker-name", "{picker}" }
                    }
                }
                a {
                    href: "{album().spotify_url}",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "review-spotify-link gap-2",
                    Icon { icon: FaSpotify }
                    "Lyssna"
                    Icon { icon: FiExternalLink }
                }
            }
        }
    }
}
