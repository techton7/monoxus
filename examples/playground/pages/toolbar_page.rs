use crate::toolbar::ToolbarPlayground;
use dioxus::prelude::*;

#[component]
pub fn ToolbarPage() -> Element {
    rsx! {
        ToolbarPlayground {}
    }
}
