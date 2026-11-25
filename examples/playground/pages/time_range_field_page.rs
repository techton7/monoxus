use crate::time_range_field::TimeRangeFieldShowcase;
use dioxus::prelude::*;

#[component]
pub fn TimeRangeFieldPage() -> Element {
    rsx! {
        TimeRangeFieldShowcase {}
    }
}
