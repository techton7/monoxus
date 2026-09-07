use dioxus::prelude::*;
use crate::tabs::TabsPlayground;

#[component]
pub fn TabsPage() -> Element {
    rsx! {
        TabsPlayground {}
    }
}
