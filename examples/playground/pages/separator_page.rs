use crate::separator::SeparatorPlayground;
use dioxus::prelude::*;

#[component]
pub fn SeparatorPage() -> Element {
    rsx! {
        SeparatorPlayground {}
    }
}
