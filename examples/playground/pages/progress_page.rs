use crate::progress::ProgressPlayground;
use dioxus::prelude::*;

#[component]
pub fn ProgressPage() -> Element {
    rsx! {
        ProgressPlayground {}
    }
}
