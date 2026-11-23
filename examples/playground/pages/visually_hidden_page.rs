use crate::visually_hidden::VisuallyHiddenPlayground;
use dioxus::prelude::*;

#[component]
pub fn VisuallyHiddenPage() -> Element {
    rsx! {
        VisuallyHiddenPlayground {}
    }
}
