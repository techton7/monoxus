use crate::rating_group::RatingGroupPlayground;
use dioxus::prelude::*;

#[component]
pub fn RatingGroupPage() -> Element {
    rsx! {
        RatingGroupPlayground {}
    }
}
