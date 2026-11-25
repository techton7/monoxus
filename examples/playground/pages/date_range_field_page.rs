use crate::date_range_field::DateRangeFieldShowcase;
use dioxus::prelude::*;

#[component]
pub fn DateRangeFieldPage() -> Element {
    rsx! {
        DateRangeFieldShowcase {}
    }
}
