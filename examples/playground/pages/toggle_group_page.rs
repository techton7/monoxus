use crate::toggle_group::ToggleGroupPlayground;
use dioxus::prelude::*;

#[component]
pub fn ToggleGroupPage() -> Element {
    rsx! {
        ToggleGroupPlayground {}
    }
}
