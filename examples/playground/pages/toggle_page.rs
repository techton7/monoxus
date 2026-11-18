use crate::toggle::TogglePlayground;
use dioxus::prelude::*;

#[component]
pub fn TogglePage() -> Element {
    rsx! {
        TogglePlayground {}
    }
}
