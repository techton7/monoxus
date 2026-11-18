use crate::switch::SwitchPlayground;
use dioxus::prelude::*;

#[component]
pub fn SwitchPage() -> Element {
    rsx! {
        SwitchPlayground {}
    }
}
