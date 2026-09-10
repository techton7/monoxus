use crate::dialog::DialogPlayground;
use dioxus::prelude::*;

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        DialogPlayground {}
    }
}
