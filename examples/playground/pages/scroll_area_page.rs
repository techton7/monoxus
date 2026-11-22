use crate::scroll_area::ScrollAreaPlayground;
use dioxus::prelude::*;

#[component]
pub fn ScrollAreaPage() -> Element {
    rsx! {
        ScrollAreaPlayground {}
    }
}
