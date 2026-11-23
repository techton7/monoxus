use crate::menubar::MenubarPlayground;
use dioxus::prelude::*;

#[component]
pub fn MenubarPage() -> Element {
    rsx! {
        MenubarPlayground {}
    }
}
