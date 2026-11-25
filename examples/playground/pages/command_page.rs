use crate::command::CommandShowcase;
use dioxus::prelude::*;

#[component]
pub fn CommandPage() -> Element {
    rsx! {
        CommandShowcase {}
    }
}
