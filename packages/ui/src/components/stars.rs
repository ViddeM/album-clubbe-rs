use dioxus::prelude::*;

const STARS_SCSS: Asset = asset!("/assets/styling/stars.scss");

/// The three possible states for an album's review score.
#[derive(Clone, PartialEq)]
pub enum ReviewScore {
    /// Reviews have not been fetched yet.
    Loading,
    /// Fetch complete; no reviews exist for this album.
    NoReviews,
    /// Fetch complete; average score and number of votes.
    Rated { avg: f32, count: usize },
}

impl ReviewScore {
    /// Compute a `ReviewScore` from a slice of raw 0–10 scores.
    pub fn from_scores(scores: &[u8]) -> Self {
        if scores.is_empty() {
            Self::NoReviews
        } else {
            let sum: u32 = scores.iter().map(|&s| s as u32).sum();
            Self::Rated {
                avg: sum as f32 / scores.len() as f32,
                count: scores.len(),
            }
        }
    }
}

#[component]
pub fn AverageStars(avg: f32, #[props(default = false)] placeholder: bool) -> Element {
    // Round to nearest integer on the 1–10 scale (each integer = one half-star)
    let display_score = avg.round() as u8;
    let class = if placeholder {
        "star-rating star-rating-readonly star-rating-placeholder"
    } else {
        "star-rating star-rating-readonly"
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STARS_SCSS }
        div {
            class: "{class}",
            "aria-label": format!("Genomsnitt {:.1} av 10", avg),
            for star in 1u8..=5 {
                {
                    let left_score = star * 2 - 1;
                    let right_score = star * 2;
                    let fill = if display_score >= right_score {
                        "full"
                    } else if display_score >= left_score {
                        "half"
                    } else {
                        "empty"
                    };
                    rsx! {
                        div { key: "{star}", class: "star-slot star-{fill}",
                            span { class: "star-bg", "★" }
                        }
                    }
                }
            }
        }
    }
}
