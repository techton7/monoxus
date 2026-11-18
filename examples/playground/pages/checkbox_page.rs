use crate::checkbox::CheckboxPlayground;
use dioxus::prelude::*;

#[component]
pub fn CheckboxPage() -> Element {
    rsx! {
        CheckboxPlayground {}
    }
}
