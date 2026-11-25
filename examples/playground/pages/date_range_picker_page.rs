use crate::date_range_picker::DateRangePickerShowcase;
use dioxus::prelude::*;

#[component]
pub fn DateRangePickerPage() -> Element {
    rsx! {
        DateRangePickerShowcase {}
    }
}
