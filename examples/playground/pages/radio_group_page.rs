use crate::radio_group::RadioGroupPlayground;
use dioxus::prelude::*;

#[component]
pub fn RadioGroupPage() -> Element {
    rsx! {
        RadioGroupPlayground {}
    }
}
