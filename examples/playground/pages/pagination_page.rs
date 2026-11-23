use crate::pagination::PaginationPlayground;
use dioxus::prelude::*;

#[component]
pub fn PaginationPage() -> Element {
    rsx! {
        PaginationPlayground {}
    }
}
