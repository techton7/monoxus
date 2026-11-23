use crate::hover_card::HoverCardPlayground;
use dioxus::prelude::*;

#[component]
pub fn HoverCardPage() -> Element {
    rsx! {
        HoverCardPlayground {}
    }
}
