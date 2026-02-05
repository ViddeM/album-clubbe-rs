use dioxus::prelude::*;

use ui::{Main, Setup};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Setup {}
        Main {}
    }
}
