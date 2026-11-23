use crate::context_menu::ContextMenuPlayground;
use dioxus::prelude::*;

#[component]
pub fn ContextMenuPage() -> Element {
    rsx! {
        ContextMenuPlayground {}
    }
}
