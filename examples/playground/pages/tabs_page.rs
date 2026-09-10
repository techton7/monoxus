use crate::tabs::TabsPlayground;
use dioxus::prelude::*;

#[component]
pub fn TabsPage() -> Element {
    rsx! {
        TabsPlayground {}
    }
}
