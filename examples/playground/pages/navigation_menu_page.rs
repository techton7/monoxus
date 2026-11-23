use crate::navigation_menu::NavigationMenuPlayground;
use dioxus::prelude::*;

#[component]
pub fn NavigationMenuPage() -> Element {
    rsx! {
        NavigationMenuPlayground {}
    }
}
