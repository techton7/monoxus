use dioxus::prelude::*;
use crate::dialog::DialogPlayground;

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        DialogPlayground {}
    }
}
