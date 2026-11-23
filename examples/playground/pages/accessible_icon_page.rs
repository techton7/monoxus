use crate::accessible_icon::AccessibleIconPlayground;
use dioxus::prelude::*;

#[component]
pub fn AccessibleIconPage() -> Element {
    rsx! {
        AccessibleIconPlayground {}
    }
}
