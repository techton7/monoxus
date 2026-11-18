use crate::button::ButtonPlayground;
use dioxus::prelude::*;

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        ButtonPlayground {}
    }
}
