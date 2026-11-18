use crate::label::LabelPlayground;
use dioxus::prelude::*;

#[component]
pub fn LabelPage() -> Element {
    rsx! {
        LabelPlayground {}
    }
}
