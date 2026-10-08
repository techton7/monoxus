use crate::collapsible::CollapsiblePlayground;
use dioxus::prelude::*;

#[component]
pub fn CollapsiblePage() -> Element {
    rsx! {
        CollapsiblePlayground {}
    }
}
