use crate::aspect_ratio::AspectRatioPlayground;
use dioxus::prelude::*;

#[component]
pub fn AspectRatioPage() -> Element {
    rsx! {
        AspectRatioPlayground {}
    }
}
