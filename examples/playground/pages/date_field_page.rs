use crate::date_field::DateFieldShowcase;
use dioxus::prelude::*;

#[component]
pub fn DateFieldPage() -> Element {
    rsx! {
        DateFieldShowcase {}
    }
}
