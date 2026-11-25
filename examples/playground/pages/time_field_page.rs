use crate::time_field::TimeFieldShowcase;
use dioxus::prelude::*;

#[component]
pub fn TimeFieldPage() -> Element {
    rsx! {
        TimeFieldShowcase {}
    }
}
