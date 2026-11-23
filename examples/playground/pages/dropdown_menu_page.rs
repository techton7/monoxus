use crate::dropdown_menu::DropdownMenuPlayground;
use dioxus::prelude::*;

#[component]
pub fn DropdownMenuPage() -> Element {
    rsx! {
        DropdownMenuPlayground {}
    }
}
